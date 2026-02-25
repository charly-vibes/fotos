use crate::capture::ImageStore;
use ab_glyph::{Font as _, FontVec, PxScale, ScaleFont as _};
use base64::Engine;
use chrono::Local;
use directories::UserDirs;
use image::Rgba;
use imageproc::drawing::{
    draw_filled_circle_mut, draw_filled_ellipse_mut, draw_hollow_ellipse_mut,
    draw_hollow_rect_mut, draw_line_segment_mut, draw_text_mut,
};
use imageproc::rect::Rect;
use serde::Deserialize;
use std::io::Cursor;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub id: String,
    #[serde(rename = "type")]
    pub annotation_type: String,
    pub x: f64,
    pub y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub stroke_color: Option<String>,
    pub fill_color: Option<String>,
    pub stroke_width: Option<f64>,
    pub opacity: Option<f64>,
    pub text: Option<String>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub points: Option<Vec<Point>>,
    pub step_number: Option<u32>,
    pub blur_radius: Option<f64>,
    pub highlight_color: Option<String>,
    pub created_at: Option<String>,
    pub locked: Option<bool>,
}

#[tauri::command]
pub fn save_image(
    image_id: String,
    annotations: Vec<Annotation>,
    _format: String,
    path: String,
    store: tauri::State<'_, ImageStore>,
) -> Result<String, String> {
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {}", e))?;

    let base_image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    let mut composite = base_image.to_rgba8();

    for anno in &annotations {
        composite_annotation(&mut composite, anno);
    }

    let (save_path, user_chosen) = if path.is_empty() {
        (generate_default_path()?, false)
    } else {
        (expand_tilde(&path)?, true)
    };

    // For auto-generated paths, guard against path traversal by requiring the path
    // stays within the home directory.  User-chosen paths (from the save dialog) are
    // already authorised by the OS portal/dialog, so we skip this check for them —
    // the dialog may return portal-translated paths like /run/user/<uid>/doc/… which
    // are outside the home directory but perfectly valid.
    if !user_chosen {
        let home = UserDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .ok_or("Could not determine home directory for path validation")?;

        let canonical = save_path
            .canonicalize()
            .unwrap_or_else(|_| save_path.clone());
        if !canonical.starts_with(&home) {
            return Err(format!(
                "Save path '{}' is outside the home directory",
                save_path.display()
            ));
        }
    }

    if let Some(parent) = save_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    composite
        .save(&save_path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(save_path.to_string_lossy().to_string())
}

/// Composite annotations onto an image and return as a base64-encoded PNG.
#[tauri::command]
pub fn composite_image(
    image_id: String,
    annotations: Vec<Annotation>,
    store: tauri::State<'_, ImageStore>,
) -> Result<String, String> {
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {}", e))?;

    let base_image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    let mut composite = base_image.to_rgba8();

    for anno in &annotations {
        composite_annotation(&mut composite, anno);
    }

    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(composite)
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(buf.into_inner()))
}

#[tauri::command]
pub fn copy_to_clipboard(
    app: tauri::AppHandle,
    image_id: String,
    annotations: Vec<Annotation>,
    store: tauri::State<'_, ImageStore>,
) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {}", e))?;

    let base_image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    let mut composite = base_image.to_rgba8();
    for anno in &annotations {
        composite_annotation(&mut composite, anno);
    }

    let (width, height) = composite.dimensions();
    let rgba_bytes = composite.into_raw();
    let image = tauri::image::Image::new_owned(rgba_bytes, width, height);

    app.clipboard()
        .write_image(&image)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))
}

#[tauri::command]
pub fn export_annotations(
    _image_id: String,
    _annotations: Vec<Annotation>,
) -> Result<String, String> {
    Err("Not yet implemented".into())
}

// ── Compositing dispatch ──────────────────────────────────────────────────────

fn composite_annotation(composite: &mut image::RgbaImage, anno: &Annotation) {
    match anno.annotation_type.as_str() {
        "rect" => composite_rectangle(composite, anno),
        "arrow" => composite_arrow(composite, anno),
        "ellipse" => composite_ellipse(composite, anno),
        "freehand" => composite_freehand(composite, anno),
        "highlight" => composite_highlight(composite, anno),
        "blur" => composite_blur(composite, anno),
        "step" => composite_step(composite, anno),
        "text" => composite_text(composite, anno),
        other => tracing::debug!("unknown annotation type: {other}"),
    }
}

// ── Per-type compositing ──────────────────────────────────────────────────────

fn composite_rectangle(composite: &mut image::RgbaImage, anno: &Annotation) {
    let x = anno.x as i32;
    let y = anno.y as i32;
    let width = anno.width.unwrap_or(0.0) as i32;
    let height = anno.height.unwrap_or(0.0) as i32;

    if width <= 0 || height <= 0 {
        return;
    }

    let opacity = (anno.opacity.unwrap_or(1.0).clamp(0.0, 1.0) * 255.0) as u8;

    // Fill (drawn first so stroke overlays it).
    let fill_str = anno.fill_color.as_deref().unwrap_or("transparent");
    if let Ok(mut fill) = parse_color(fill_str) {
        if fill[3] > 0 {
            fill[3] = ((fill[3] as f64 / 255.0) * opacity as f64) as u8;
            for py in y..(y + height) {
                for px in x..(x + width) {
                    if px >= 0
                        && py >= 0
                        && (px as u32) < composite.width()
                        && (py as u32) < composite.height()
                    {
                        let base = composite.get_pixel_mut(px as u32, py as u32);
                        blend_pixel(base, fill);
                    }
                }
            }
        }
    }

    // Stroke — centered on the rect edge (half inside, half outside).
    let stroke_color_str = anno.stroke_color.as_deref().unwrap_or("#FF0000");
    let Ok(stroke_color) = parse_color(stroke_color_str) else {
        return;
    };
    if stroke_color[3] == 0 {
        return;
    }

    let sw = anno.stroke_width.unwrap_or(2.0).max(0.0) as i32;
    let half = sw / 2;

    for i in 0..sw {
        let offset = i - half;
        let rx = x - offset;
        let ry = y - offset;
        let rw = width + offset * 2;
        let rh = height + offset * 2;

        if rw <= 0 || rh <= 0 {
            continue;
        }

        let rect = Rect::at(rx, ry).of_size(rw as u32, rh as u32);
        draw_hollow_rect_mut(composite, rect, stroke_color);
    }
}

fn composite_arrow(composite: &mut image::RgbaImage, anno: &Annotation) {
    let points = match &anno.points {
        Some(pts) if pts.len() >= 2 => pts,
        _ => return,
    };
    let stroke_str = anno.stroke_color.as_deref().unwrap_or("#FF0000");
    let Ok(color) = parse_color(stroke_str) else { return };
    let sw = anno.stroke_width.unwrap_or(2.0).max(1.0);

    let p1 = &points[0];
    let p2 = &points[1];

    // Shaft.
    draw_thick_line(
        composite,
        p1.x as f32, p1.y as f32,
        p2.x as f32, p2.y as f32,
        color, sw as f32,
    );

    // Arrowhead — two lines from tip.
    let head_len = (sw * 5.0).max(12.0);
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }
    let angle = dy.atan2(dx);
    let wing = std::f64::consts::PI / 6.0;

    let wx1 = (p2.x - head_len * (angle - wing).cos()) as f32;
    let wy1 = (p2.y - head_len * (angle - wing).sin()) as f32;
    let wx2 = (p2.x - head_len * (angle + wing).cos()) as f32;
    let wy2 = (p2.y - head_len * (angle + wing).sin()) as f32;

    draw_thick_line(composite, p2.x as f32, p2.y as f32, wx1, wy1, color, sw as f32);
    draw_thick_line(composite, p2.x as f32, p2.y as f32, wx2, wy2, color, sw as f32);
}

fn composite_ellipse(composite: &mut image::RgbaImage, anno: &Annotation) {
    let x = anno.x as i32;
    let y = anno.y as i32;
    let width = anno.width.unwrap_or(0.0) as i32;
    let height = anno.height.unwrap_or(0.0) as i32;

    if width <= 0 || height <= 0 {
        return;
    }

    let cx = x + width / 2;
    let cy = y + height / 2;
    let rx = width / 2;
    let ry = height / 2;

    let opacity = anno.opacity.unwrap_or(1.0).clamp(0.0, 1.0);

    // Fill.
    let fill_str = anno.fill_color.as_deref().unwrap_or("transparent");
    if let Ok(mut fill) = parse_color(fill_str) {
        if fill[3] > 0 {
            fill[3] = (fill[3] as f64 * opacity) as u8;
            draw_filled_ellipse_mut(composite, (cx, cy), rx, ry, fill);
        }
    }

    // Stroke.
    let stroke_str = anno.stroke_color.as_deref().unwrap_or("#FF0000");
    if let Ok(mut stroke) = parse_color(stroke_str) {
        if stroke[3] > 0 {
            stroke[3] = (stroke[3] as f64 * opacity) as u8;
            draw_hollow_ellipse_mut(composite, (cx, cy), rx, ry, stroke);
        }
    }
}

fn composite_freehand(composite: &mut image::RgbaImage, anno: &Annotation) {
    let points = match &anno.points {
        Some(pts) if pts.len() >= 2 => pts,
        _ => return,
    };
    let stroke_str = anno.stroke_color.as_deref().unwrap_or("#FF0000");
    let Ok(color) = parse_color(stroke_str) else { return };
    let sw = anno.stroke_width.unwrap_or(2.0).max(1.0);

    for i in 0..points.len() - 1 {
        let p1 = &points[i];
        let p2 = &points[i + 1];
        draw_thick_line(
            composite,
            p1.x as f32, p1.y as f32,
            p2.x as f32, p2.y as f32,
            color, sw as f32,
        );
    }
}

fn composite_highlight(composite: &mut image::RgbaImage, anno: &Annotation) {
    let x = anno.x as i32;
    let y = anno.y as i32;
    let width = anno.width.unwrap_or(0.0) as i32;
    let height = anno.height.unwrap_or(0.0) as i32;

    if width <= 0 || height <= 0 {
        return;
    }

    let color_str = anno.highlight_color.as_deref().unwrap_or("#FFFF00");
    let Ok(mut color) = parse_color(color_str) else { return };
    // Always 0.4 opacity per spec.
    color[3] = (0.4 * 255.0) as u8;

    let img_w = composite.width() as i32;
    let img_h = composite.height() as i32;

    for py in y..(y + height) {
        for px in x..(x + width) {
            if px >= 0 && py >= 0 && px < img_w && py < img_h {
                let base = composite.get_pixel_mut(px as u32, py as u32);
                blend_pixel(base, color);
            }
        }
    }
}

fn composite_blur(composite: &mut image::RgbaImage, anno: &Annotation) {
    let x = anno.x as u32;
    let y = anno.y as u32;
    let w = anno.width.unwrap_or(0.0) as u32;
    let h = anno.height.unwrap_or(0.0) as u32;
    let block_size = anno.blur_radius.unwrap_or(10.0).max(1.0) as u32;

    if w == 0 || h == 0 {
        return;
    }

    let img_w = composite.width();
    let img_h = composite.height();

    let x2 = (x + w).min(img_w);
    let y2 = (y + h).min(img_h);
    let x1 = x.min(x2);
    let y1 = y.min(y2);

    let mut bx = x1;
    while bx < x2 {
        let bx2 = (bx + block_size).min(x2);
        let mut by = y1;
        while by < y2 {
            let by2 = (by + block_size).min(y2);
            let count = (bx2 - bx) * (by2 - by);

            let mut r_sum = 0u32;
            let mut g_sum = 0u32;
            let mut b_sum = 0u32;
            let mut a_sum = 0u32;

            for py in by..by2 {
                for px in bx..bx2 {
                    let p = composite.get_pixel(px, py);
                    r_sum += p[0] as u32;
                    g_sum += p[1] as u32;
                    b_sum += p[2] as u32;
                    a_sum += p[3] as u32;
                }
            }

            let avg = Rgba([
                (r_sum / count) as u8,
                (g_sum / count) as u8,
                (b_sum / count) as u8,
                (a_sum / count) as u8,
            ]);

            for py in by..by2 {
                for px in bx..bx2 {
                    composite.put_pixel(px, py, avg);
                }
            }

            by += block_size;
        }
        bx += block_size;
    }
}

fn composite_step(composite: &mut image::RgbaImage, anno: &Annotation) {
    let cx = anno.x as i32;
    let cy = anno.y as i32;
    let size = anno.font_size.unwrap_or(24.0) as i32;
    let radius = size / 2;

    let stroke_str = anno.stroke_color.as_deref().unwrap_or("#FF0000");
    let Ok(color) = parse_color(stroke_str) else { return };

    draw_filled_circle_mut(composite, (cx, cy), radius, color);

    // Draw the step number centered inside the circle.
    let step = match anno.step_number {
        Some(n) => n,
        None => return,
    };
    let font = embedded_font();
    let text = step.to_string();
    let font_size = (size as f32 * 0.6).max(8.0);
    let scale = PxScale { x: font_size, y: font_size };
    let scaled = font.as_scaled(scale);
    let text_width: f32 = text.chars().map(|c| scaled.h_advance(font.glyph_id(c))).sum();
    let ascent = scaled.ascent();
    let tx = cx - (text_width / 2.0) as i32;
    let ty = cy - (ascent / 2.0) as i32;
    draw_text_mut(composite, Rgba([255, 255, 255, 255]), tx, ty, scale, &font, &text);
}

fn embedded_font() -> FontVec {
    static BYTES: &[u8] =
        include_bytes!("../../fonts/LiberationSans-Regular.ttf");
    FontVec::try_from_vec(BYTES.to_vec()).expect("embedded font is valid")
}

fn composite_text(composite: &mut image::RgbaImage, anno: &Annotation) {
    let text = match &anno.text {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };

    let font = embedded_font();
    let x = anno.x as i32;
    let y = anno.y as i32;
    let font_size = anno.font_size.unwrap_or(20.0) as f32;
    let scale = PxScale { x: font_size, y: font_size };

    let stroke_str = anno.stroke_color.as_deref().unwrap_or("#FF0000");
    let Ok(color) = parse_color(stroke_str) else { return };

    let line_height = (font_size * 1.4) as i32;
    for (i, line) in text.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let line_y = y + (i as i32 * line_height);
        draw_text_mut(composite, color, x, line_y, scale, &font, line);
    }
}

// ── Line helpers ──────────────────────────────────────────────────────────────

/// Draw a line with approximate stroke width by drawing parallel offset lines.
fn draw_thick_line(
    composite: &mut image::RgbaImage,
    x1: f32, y1: f32,
    x2: f32, y2: f32,
    color: Rgba<u8>,
    width: f32,
) {
    if width <= 1.0 {
        draw_line_segment_mut(composite, (x1, y1), (x2, y2), color);
        return;
    }

    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }
    // Perpendicular unit vector.
    let px = -dy / len;
    let py = dx / len;

    let half = width / 2.0;
    let steps = width.ceil() as i32;

    for i in 0..=steps {
        let t = ((i as f32 / steps.max(1) as f32) - 0.5) * width;
        let t = t.clamp(-half, half);
        draw_line_segment_mut(
            composite,
            (x1 + px * t, y1 + py * t),
            (x2 + px * t, y2 + py * t),
            color,
        );
    }
}

// ── Shared pixel helpers ──────────────────────────────────────────────────────

/// Alpha-composite `src` over `dst` in place (src-over).
#[inline]
fn blend_pixel(dst: &mut Rgba<u8>, src: Rgba<u8>) {
    let sa = src[3] as f64 / 255.0;
    let da = dst[3] as f64 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a < f64::EPSILON {
        *dst = Rgba([0, 0, 0, 0]);
        return;
    }
    dst[0] = ((src[0] as f64 * sa + dst[0] as f64 * da * (1.0 - sa)) / out_a) as u8;
    dst[1] = ((src[1] as f64 * sa + dst[1] as f64 * da * (1.0 - sa)) / out_a) as u8;
    dst[2] = ((src[2] as f64 * sa + dst[2] as f64 * da * (1.0 - sa)) / out_a) as u8;
    dst[3] = (out_a * 255.0) as u8;
}

/// Parse a CSS color string into `Rgba<u8>`.
///
/// Supports:
/// - `"transparent"` → `[0, 0, 0, 0]`
/// - `#RRGGBB` → alpha 255
/// - `#RRGGBBAA` → explicit alpha
///
/// Returns `Err` for unrecognized formats.
fn parse_color(color: &str) -> Result<Rgba<u8>, String> {
    if color.eq_ignore_ascii_case("transparent") {
        return Ok(Rgba([0, 0, 0, 0]));
    }

    let hex = color
        .strip_prefix('#')
        .ok_or_else(|| format!("Unsupported color format: {color}"))?;

    let parse_byte = |s: &str| {
        u8::from_str_radix(s, 16).map_err(|_| format!("Invalid hex byte '{s}' in color {color}"))
    };

    match hex.len() {
        6 => Ok(Rgba([
            parse_byte(&hex[0..2])?,
            parse_byte(&hex[2..4])?,
            parse_byte(&hex[4..6])?,
            255,
        ])),
        8 => Ok(Rgba([
            parse_byte(&hex[0..2])?,
            parse_byte(&hex[2..4])?,
            parse_byte(&hex[4..6])?,
            parse_byte(&hex[6..8])?,
        ])),
        _ => Err(format!("Unsupported color format: {color}")),
    }
}

fn generate_default_path() -> Result<PathBuf, String> {
    let user_dirs = UserDirs::new().ok_or("Could not find user directories")?;
    let pictures = user_dirs
        .picture_dir()
        .ok_or("Could not find Pictures directory")?;

    let fotos_dir = pictures.join("Fotos");
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("fotos-{}.png", timestamp);

    Ok(fotos_dir.join(filename))
}

fn expand_tilde(path: &str) -> Result<PathBuf, String> {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = UserDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .ok_or("Could not resolve home directory for tilde expansion")?;
        return Ok(home.join(stripped));
    }
    Ok(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn make_annotation(
        ann_type: &str,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        stroke_color: Option<&str>,
        fill_color: Option<&str>,
        stroke_width: Option<f64>,
        opacity: Option<f64>,
    ) -> Annotation {
        Annotation {
            id: "test".into(),
            annotation_type: ann_type.into(),
            x,
            y,
            width: Some(w),
            height: Some(h),
            stroke_color: stroke_color.map(String::from),
            fill_color: fill_color.map(String::from),
            stroke_width,
            opacity,
            text: None,
            font_size: None,
            font_family: None,
            points: None,
            step_number: None,
            blur_radius: None,
            highlight_color: None,
            created_at: None,
            locked: None,
        }
    }

    // ── parse_color ──────────────────────────────────────────────────────────

    #[test]
    fn parse_color_transparent() {
        assert_eq!(parse_color("transparent").unwrap(), Rgba([0, 0, 0, 0]));
        assert_eq!(parse_color("TRANSPARENT").unwrap(), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn parse_color_rrggbb() {
        assert_eq!(parse_color("#ff0000").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("#00FF00").unwrap(), Rgba([0, 255, 0, 255]));
    }

    #[test]
    fn parse_color_rrggbbaa() {
        assert_eq!(parse_color("#ff000080").unwrap(), Rgba([255, 0, 0, 128]));
    }

    #[test]
    fn parse_color_invalid_returns_err() {
        assert!(parse_color("red").is_err());
        assert!(parse_color("#zzz").is_err());
        assert!(parse_color("").is_err());
    }

    // ── compositing ──────────────────────────────────────────────────────────

    #[test]
    fn compositing_stroke_pixel_is_stroke_color() {
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let anno = make_annotation(
            "rect", 10.0, 10.0, 30.0, 20.0,
            Some("#ff0000"), Some("transparent"), Some(1.0), Some(1.0),
        );
        composite_rectangle(&mut img, &anno);
        assert_eq!(*img.get_pixel(10, 10), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn compositing_outside_pixel_unchanged() {
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let anno = make_annotation(
            "rect", 10.0, 10.0, 30.0, 20.0,
            Some("#ff0000"), Some("transparent"), Some(1.0), Some(1.0),
        );
        composite_rectangle(&mut img, &anno);
        assert_eq!(*img.get_pixel(0, 0), Rgba([255, 255, 255, 255]));
        assert_eq!(*img.get_pixel(99, 99), Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn compositing_fill_pixel_is_fill_color() {
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let anno = make_annotation(
            "rect", 10.0, 10.0, 30.0, 20.0,
            Some("transparent"), Some("#0000ff"), Some(0.0), Some(1.0),
        );
        composite_rectangle(&mut img, &anno);
        let px = img.get_pixel(20, 15);
        assert_eq!(px[2], 255);
        assert_eq!(px[0], 0);
    }

    #[test]
    fn compositing_highlight_uses_fixed_opacity() {
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 0, 255]));
        let mut anno = make_annotation(
            "highlight", 10.0, 10.0, 20.0, 20.0,
            None, None, None, Some(1.0),
        );
        anno.highlight_color = Some("#FFFF00".into());
        composite_highlight(&mut img, &anno);
        // The yellow should be blended at 0.4 opacity over black.
        let px = img.get_pixel(20, 20);
        assert!(px[0] > 0, "yellow channel should be non-zero after highlight");
        assert!(px[0] < 255, "should be blended, not fully opaque");
    }

    #[test]
    fn compositing_blur_pixelates_region() {
        // Fill top-left 20×20 with red, rest with blue.
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 255, 255]));
        for y in 0..20u32 {
            for x in 0..20u32 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let mut anno = make_annotation(
            "blur", 0.0, 0.0, 20.0, 20.0,
            None, None, None, None,
        );
        anno.blur_radius = Some(10.0);
        composite_blur(&mut img, &anno);
        // After blur, the 10×10 blocks should all have the same average red color.
        let block1 = *img.get_pixel(0, 0);
        let block2 = *img.get_pixel(5, 5);
        assert_eq!(block1, block2, "pixels within a blur block should be identical");
    }

    #[test]
    fn compositing_arrow_draws_something() {
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let mut anno = make_annotation(
            "arrow", 0.0, 0.0, 0.0, 0.0,
            Some("#ff0000"), None, Some(2.0), Some(1.0),
        );
        anno.points = Some(vec![
            Point { x: 10.0, y: 50.0 },
            Point { x: 80.0, y: 50.0 },
        ]);
        composite_arrow(&mut img, &anno);
        // At least the shaft midpoint should be red.
        let mid = img.get_pixel(45, 50);
        assert_eq!(mid[0], 255, "arrow shaft should be red");
        assert_eq!(mid[1], 0);
    }

    // ── Annotation serde (camelCase round-trip) ──────────────────────────────

    #[test]
    fn annotation_serde_camel_case_round_trip() {
        let json = r##"{
            "id": "abc",
            "type": "rect",
            "x": 5.0,
            "y": 10.0,
            "width": 100.0,
            "height": 50.0,
            "strokeColor": "#ff0000",
            "fillColor": "transparent",
            "strokeWidth": 2.0,
            "opacity": 0.8
        }"##;

        let anno: Annotation = serde_json::from_str(json).expect("deserialization failed");
        assert_eq!(anno.annotation_type, "rect");
        assert_eq!(anno.stroke_color.as_deref(), Some("#ff0000"));
        assert_eq!(anno.fill_color.as_deref(), Some("transparent"));
        assert_eq!(anno.stroke_width, Some(2.0));
        assert_eq!(anno.opacity, Some(0.8));
    }

    #[test]
    fn annotation_serde_new_fields() {
        let json = r##"{
            "id": "def",
            "type": "step",
            "x": 50.0,
            "y": 50.0,
            "strokeColor": "#ff0000",
            "stepNumber": 3,
            "fontSize": 24.0,
            "highlightColor": "#FFFF00",
            "blurRadius": 10.0
        }"##;

        let anno: Annotation = serde_json::from_str(json).expect("deserialization failed");
        assert_eq!(anno.step_number, Some(3));
        assert_eq!(anno.blur_radius, Some(10.0));
        assert_eq!(anno.highlight_color.as_deref(), Some("#FFFF00"));
    }
}
