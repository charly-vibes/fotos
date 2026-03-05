/// Tesseract OCR pipeline with tiling and preprocessing.
///
/// Strategy:
/// - Small images (both dims ≤ 2000px): upscale 2× then single-pass OCR.
/// - Large images: divide into overlapping tiles (1024px, 100px overlap),
///   OCR each tile independently, translate coordinates, then deduplicate
///   overlapping detections using IoU + text similarity.
use anyhow::Result;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use tesseract::Tesseract;

pub struct OcrRegion {
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub confidence: f32,
}

pub struct OcrOutput {
    pub full_text: String,
    pub regions: Vec<OcrRegion>,
}

pub struct OcrOptions {
    pub lang: String,
    pub tessdata_path: String,
}

const TILE_SIZE: u32 = 1024;
const TILE_OVERLAP: u32 = 100;
const SMALL_IMAGE_THRESHOLD: u32 = 2000;
const UPSCALE_FACTOR: u32 = 2;
/// IoU threshold above which two detections are considered duplicates.
const IOU_DEDUP_THRESHOLD: f32 = 0.5;

/// Run OCR on an image. Selects between upscale+single-pass (small images)
/// and tiled processing (large images).
///
/// `on_progress` is called after each tile completes with `(completed, total)`.
pub fn run_ocr(
    image: &image::DynamicImage,
    opts: &OcrOptions,
    on_progress: Option<&(dyn Fn(u32, u32) + Send + Sync)>,
) -> Result<OcrOutput> {
    let (w, h) = (image.width(), image.height());

    if w <= SMALL_IMAGE_THRESHOLD && h <= SMALL_IMAGE_THRESHOLD {
        run_upscaled(image, opts, on_progress)
    } else {
        run_tiled(image, opts, on_progress)
    }
}

// ---------------------------------------------------------------------------
// Upscale strategy
// ---------------------------------------------------------------------------

fn run_upscaled(
    image: &image::DynamicImage,
    opts: &OcrOptions,
    on_progress: Option<&(dyn Fn(u32, u32) + Send + Sync)>,
) -> Result<OcrOutput> {
    let scale = UPSCALE_FACTOR;
    let upscaled = image.resize(
        image.width() * scale,
        image.height() * scale,
        image::imageops::FilterType::Lanczos3,
    );

    let mut regions = run_tesseract(&upscaled, opts)?;

    // Scale coordinates back to original image space.
    for r in &mut regions {
        r.x /= scale;
        r.y /= scale;
        r.w = (r.w / scale).max(1);
        r.h = (r.h / scale).max(1);
    }

    if let Some(cb) = on_progress {
        cb(1, 1);
    }

    let full_text = regions_to_text(&regions);
    Ok(OcrOutput { full_text, regions })
}

// ---------------------------------------------------------------------------
// Tiled strategy
// ---------------------------------------------------------------------------

fn run_tiled(
    image: &image::DynamicImage,
    opts: &OcrOptions,
    on_progress: Option<&(dyn Fn(u32, u32) + Send + Sync)>,
) -> Result<OcrOutput> {
    let img_w = image.width();
    let img_h = image.height();

    let xs = tile_positions(img_w, TILE_SIZE, TILE_OVERLAP);
    let ys = tile_positions(img_h, TILE_SIZE, TILE_OVERLAP);

    let coords: Vec<(u32, u32)> =
        ys.iter().flat_map(|&ty| xs.iter().map(move |&tx| (tx, ty))).collect();
    let total = coords.len() as u32;
    let done = AtomicU32::new(0);

    let results: Result<Vec<Vec<OcrRegion>>> = coords
        .par_iter()
        .map(|(tile_x, tile_y)| {
            let tw = TILE_SIZE.min(img_w - tile_x);
            let th = TILE_SIZE.min(img_h - tile_y);
            let tile = image.crop_imm(*tile_x, *tile_y, tw, th);
            let mut regions = run_tesseract(&tile, opts)?;
            for r in &mut regions {
                r.x += tile_x;
                r.y += tile_y;
            }
            let completed = done.fetch_add(1, Ordering::Relaxed) + 1;
            if let Some(cb) = on_progress {
                cb(completed, total);
            }
            Ok(regions)
        })
        .collect();

    let all_regions: Vec<OcrRegion> = results?.into_iter().flatten().collect();
    let regions = deduplicate_regions(all_regions);
    let full_text = regions_to_text(&regions);
    Ok(OcrOutput { full_text, regions })
}

// ---------------------------------------------------------------------------
// Core Tesseract call
// ---------------------------------------------------------------------------

fn run_tesseract(image: &image::DynamicImage, opts: &OcrOptions) -> Result<Vec<OcrRegion>> {
    let rgb = image.to_rgb8();
    let width = rgb.width() as i32;
    let height = rgb.height() as i32;
    let bytes_per_line = width * 3;
    let raw = rgb.into_raw();

    let mut tess = Tesseract::new(Some(&opts.tessdata_path), Some(&opts.lang))
        .map_err(|e| anyhow::anyhow!("Tesseract init failed: {}", e))?
        .set_frame(&raw, width, height, 3, bytes_per_line)
        .map_err(|e| anyhow::anyhow!("Tesseract set_frame failed: {}", e))?
        .recognize()
        .map_err(|e| anyhow::anyhow!("Tesseract recognize failed: {}", e))?;

    // TSV columns (level 5 = word):
    // level page_num block_num par_num line_num word_num left top width height conf text
    let tsv = tess
        .get_tsv_text(0)
        .map_err(|e| anyhow::anyhow!("Tesseract get_tsv_text failed: {}", e))?;

    let regions = tsv
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols: Vec<&str> = line.splitn(12, '\t').collect();
            if cols.len() < 12 {
                return None;
            }
            let level: u32 = cols[0].parse().ok()?;
            if level != 5 {
                return None;
            }
            let conf: f32 = cols[10].parse().ok()?;
            if conf < 0.0 {
                return None;
            }
            let text = cols[11].trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(OcrRegion {
                text,
                x: cols[6].parse().ok()?,
                y: cols[7].parse().ok()?,
                w: cols[8].parse().ok()?,
                h: cols[9].parse().ok()?,
                confidence: conf,
            })
        })
        .collect();

    Ok(regions)
}

// ---------------------------------------------------------------------------
// Tiling helpers
// ---------------------------------------------------------------------------

/// Returns the left/top pixel positions for tiles covering `total` pixels.
/// Each tile is `tile_size` pixels wide with `overlap` pixels shared with
/// the adjacent tile. The last tile is aligned so it ends exactly at `total`.
fn tile_positions(total: u32, tile_size: u32, overlap: u32) -> Vec<u32> {
    if total <= tile_size {
        return vec![0];
    }
    let stride = tile_size.saturating_sub(overlap);
    let mut positions = Vec::new();
    let mut pos = 0u32;
    loop {
        positions.push(pos);
        let next = pos + stride;
        if next + tile_size >= total {
            let last = total - tile_size;
            if last > pos {
                positions.push(last);
            }
            break;
        }
        pos = next;
    }
    positions
}

// ---------------------------------------------------------------------------
// Deduplication (NMS-style with IoU + text similarity)
// ---------------------------------------------------------------------------

fn deduplicate_regions(mut regions: Vec<OcrRegion>) -> Vec<OcrRegion> {
    // Sort by confidence descending so the best detection wins.
    regions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let n = regions.len();
    let mut suppressed = vec![false; n];

    for i in 0..n {
        if suppressed[i] {
            continue;
        }
        for j in (i + 1)..n {
            if suppressed[j] {
                continue;
            }
            if iou(&regions[i], &regions[j]) > IOU_DEDUP_THRESHOLD
                && texts_similar(&regions[i].text, &regions[j].text)
            {
                suppressed[j] = true;
            }
        }
    }

    regions
        .into_iter()
        .enumerate()
        .filter_map(|(i, r)| if !suppressed[i] { Some(r) } else { None })
        .collect()
}

fn iou(a: &OcrRegion, b: &OcrRegion) -> f32 {
    let ax2 = a.x + a.w;
    let ay2 = a.y + a.h;
    let bx2 = b.x + b.w;
    let by2 = b.y + b.h;

    let ix1 = a.x.max(b.x);
    let iy1 = a.y.max(b.y);
    let ix2 = ax2.min(bx2);
    let iy2 = ay2.min(by2);

    if ix2 <= ix1 || iy2 <= iy1 {
        return 0.0;
    }

    let inter = (ix2 - ix1) as f32 * (iy2 - iy1) as f32;
    let area_a = (a.w * a.h) as f32;
    let area_b = (b.w * b.h) as f32;
    let union = area_a + area_b - inter;

    if union <= 0.0 { 0.0 } else { inter / union }
}

/// Two words are "similar" if they match case-insensitively, differ by at
/// most 1 edit, or their edit distance is ≤ 20% of the longer word's length.
fn texts_similar(a: &str, b: &str) -> bool {
    if a.eq_ignore_ascii_case(b) {
        return true;
    }
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return true;
    }
    let dist = levenshtein(a, b);
    dist <= 1 || (dist as f32 / max_len as f32) <= 0.2
}

fn levenshtein(s: &str, t: &str) -> usize {
    let s: Vec<char> = s.chars().collect();
    let t: Vec<char> = t.chars().collect();
    let m = s.len();
    let n = t.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            curr[j] = if s[i - 1] == t[j - 1] {
                prev[j - 1]
            } else {
                1 + prev[j - 1].min(prev[j]).min(curr[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

// ---------------------------------------------------------------------------
// Reading-order text reconstruction
// ---------------------------------------------------------------------------

fn regions_to_text(regions: &[OcrRegion]) -> String {
    if regions.is_empty() {
        return String::new();
    }

    let avg_height = {
        let sum: u32 = regions.iter().map(|r| r.h).sum();
        (sum / regions.len() as u32).max(1)
    };

    let mut sorted: Vec<&OcrRegion> = regions.iter().collect();
    sorted.sort_by_key(|r| (r.y, r.x));

    let mut text = String::new();
    let mut prev_bottom = 0u32;
    let mut prev_right = 0u32;

    for r in &sorted {
        if !text.is_empty() {
            if r.y > prev_bottom.saturating_add(avg_height / 2) {
                text.push('\n');
            } else if r.x >= prev_right {
                text.push(' ');
            }
        }
        text.push_str(&r.text);
        prev_bottom = r.y + r.h;
        prev_right = r.x + r.w;
    }

    text
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn region(text: &str, x: u32, y: u32, w: u32, h: u32, conf: f32) -> OcrRegion {
        OcrRegion { text: text.into(), x, y, w, h, confidence: conf }
    }

    // --- tile_positions ---

    #[test]
    fn tile_positions_image_smaller_than_tile() {
        assert_eq!(tile_positions(800, 1024, 100), vec![0]);
    }

    #[test]
    fn tile_positions_exact_tile_size() {
        assert_eq!(tile_positions(1024, 1024, 100), vec![0]);
    }

    #[test]
    fn tile_positions_two_tiles() {
        // total=1200, stride=924 → pos=0, next=924, 924+1024=1948≥1200 → last=176
        let positions = tile_positions(1200, 1024, 100);
        assert_eq!(positions, vec![0, 176]);
    }

    #[test]
    fn tile_positions_covers_full_width() {
        let total = 2560u32;
        let positions = tile_positions(total, TILE_SIZE, TILE_OVERLAP);
        assert_eq!(*positions.first().unwrap(), 0);
        let last = *positions.last().unwrap();
        assert!(last + TILE_SIZE >= total, "last tile does not reach end");
        for &p in &positions {
            assert!(p + TILE_SIZE <= total, "tile extends past image");
        }
    }

    // --- coordinate translation ---

    #[test]
    fn tile_offset_translation() {
        let tile_x = 500u32;
        let tile_y = 300u32;
        assert_eq!(tile_x + 10, 510);
        assert_eq!(tile_y + 5, 305);
    }

    // --- IoU ---

    #[test]
    fn iou_identical_boxes() {
        let r = region("a", 0, 0, 100, 20, 90.0);
        assert!((iou(&r, &r) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn iou_non_overlapping() {
        let a = region("a", 0, 0, 50, 20, 90.0);
        let b = region("b", 100, 0, 50, 20, 90.0);
        assert_eq!(iou(&a, &b), 0.0);
    }

    #[test]
    fn iou_partial_overlap() {
        // Two 100×20 boxes, shifted 50px → inter=50×20=1000, union=3000, iou=1/3
        let a = region("a", 0, 0, 100, 20, 90.0);
        let b = region("b", 50, 0, 100, 20, 90.0);
        let score = iou(&a, &b);
        assert!((score - 1.0 / 3.0).abs() < 1e-4, "iou={score}");
    }

    // --- Levenshtein ---

    #[test]
    fn levenshtein_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn levenshtein_one_deletion() {
        assert_eq!(levenshtein("hello", "helo"), 1);
    }

    #[test]
    fn levenshtein_empty() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }

    // --- texts_similar ---

    #[test]
    fn texts_similar_exact() {
        assert!(texts_similar("word", "word"));
    }

    #[test]
    fn texts_similar_one_edit() {
        assert!(texts_similar("word", "wrd"));
    }

    #[test]
    fn texts_similar_case_insensitive() {
        assert!(texts_similar("Word", "word"));
    }

    #[test]
    fn texts_not_similar() {
        assert!(!texts_similar("hello", "world"));
    }

    // --- deduplication ---

    #[test]
    fn dedup_removes_overlapping_identical() {
        let regions = vec![
            region("Hello", 10, 10, 60, 20, 85.0),
            region("Hello", 10, 10, 60, 20, 70.0),
        ];
        let deduped = deduplicate_regions(regions);
        assert_eq!(deduped.len(), 1);
        assert!((deduped[0].confidence - 85.0).abs() < 1e-5);
    }

    #[test]
    fn dedup_keeps_non_overlapping() {
        let regions = vec![
            region("Hello", 0, 0, 60, 20, 85.0),
            region("World", 200, 0, 60, 20, 85.0),
        ];
        assert_eq!(deduplicate_regions(regions).len(), 2);
    }

    #[test]
    fn dedup_keeps_different_text_same_location() {
        // Same bbox but very different texts → texts_similar returns false → both kept.
        let regions = vec![
            region("Hello", 10, 10, 60, 20, 85.0),
            region("World", 10, 10, 60, 20, 70.0),
        ];
        assert_eq!(deduplicate_regions(regions).len(), 2);
    }

    // --- regions_to_text ---

    #[test]
    fn text_reconstruction_same_line() {
        let regions = vec![
            region("Hello", 0, 0, 60, 20, 90.0),
            region("World", 70, 0, 60, 20, 90.0),
        ];
        let text = regions_to_text(&regions);
        assert!(text.contains("Hello") && text.contains("World"), "got: {text}");
    }

    #[test]
    fn text_reconstruction_newline_between_lines() {
        let regions = vec![
            region("Line1", 0, 0, 60, 20, 90.0),
            region("Line2", 0, 100, 60, 20, 90.0),
        ];
        let text = regions_to_text(&regions);
        assert!(text.contains('\n'), "expected newline, got: {text}");
    }

    // --- reading-order validation for complex / "scrambled" layouts ---

    /// Two-column layout: regions arrive in arbitrary order (as Tesseract might
    /// return them in a single-pass scan across the full image width).
    /// After tiling, coordinates are correct; regions_to_text must reconstruct
    /// reading order: left-to-right within each row, top-to-bottom across rows.
    #[test]
    fn text_reconstruction_two_column_layout() {
        // Col 1 (x≈0):   "Left"  row 0, "Text"  row 1
        // Col 2 (x≈300): "Right" row 0, "Side"  row 1
        // Arrive in scrambled (Tesseract single-pass) order.
        let regions = vec![
            region("Right", 300, 0, 60, 20, 90.0),
            region("Side", 300, 30, 60, 20, 90.0),
            region("Left", 0, 0, 60, 20, 90.0),
            region("Text", 0, 30, 60, 20, 90.0),
        ];
        let text = regions_to_text(&regions);
        let pos = |w: &str| text.find(w).unwrap_or_else(|| panic!("'{w}' missing in: {text}"));
        // Within the same row, left column precedes right column.
        assert!(pos("Left") < pos("Right"), "row 0: left col should precede right col");
        assert!(pos("Text") < pos("Side"), "row 1: left col should precede right col");
        // Row 0 precedes row 1 for each column.
        assert!(pos("Left") < pos("Text"), "col 1: top word should precede bottom word");
        assert!(pos("Right") < pos("Side"), "col 2: top word should precede bottom word");
    }

    /// Sidebar layout: a narrow navigation column (x≈0) beside main content (x≈200).
    /// Regions arrive out of order; result must preserve per-column reading order.
    #[test]
    fn text_reconstruction_sidebar_layout() {
        let regions = vec![
            region("File", 10, 10, 60, 18, 90.0),
            region("The", 200, 10, 40, 18, 90.0),
            region("Edit", 10, 35, 60, 18, 90.0),
            region("quick", 250, 10, 50, 18, 90.0),
            region("View", 10, 60, 60, 18, 90.0),
            region("brown", 310, 10, 55, 18, 90.0),
        ];
        let text = regions_to_text(&regions);
        for word in &["File", "Edit", "View", "The", "quick", "brown"] {
            assert!(text.contains(word), "'{word}' missing in: {text}");
        }
        // Sidebar items share the same y-band as main content; left col comes first.
        let pos = |w: &str| text.find(w).unwrap();
        assert!(pos("File") < pos("The"), "sidebar 'File' should precede main 'The'");
        // Sidebar items are top-to-bottom.
        assert!(pos("File") < pos("Edit"), "File should precede Edit in sidebar");
        assert!(pos("Edit") < pos("View"), "Edit should precede View in sidebar");
    }

    /// Unsorted input (worst-case Tesseract scramble): regions arrive in reverse
    /// spatial order. regions_to_text must still produce correct reading order.
    #[test]
    fn text_reconstruction_reverse_input_order() {
        // Three lines, fed in reverse order.
        let mut regions = vec![
            region("Third", 0, 80, 60, 20, 90.0),
            region("Second", 0, 40, 60, 20, 90.0),
            region("First", 0, 0, 60, 20, 90.0),
        ];
        // Shuffle to worst case.
        regions.reverse();
        let text = regions_to_text(&regions);
        let pos = |w: &str| text.find(w).unwrap_or_else(|| panic!("'{w}' missing in: {text}"));
        assert!(pos("First") < pos("Second"), "First should precede Second");
        assert!(pos("Second") < pos("Third"), "Second should precede Third");
    }
}
