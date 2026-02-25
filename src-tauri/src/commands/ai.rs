use crate::capture::ImageStore;
use serde::Serialize;
use std::path::PathBuf;
use tesseract::Tesseract;
use uuid::Uuid;

#[derive(Serialize)]
pub struct OcrRegion {
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub confidence: f32,
}

#[derive(Serialize)]
pub struct OcrResult {
    pub text: String,
    pub regions: Vec<OcrRegion>,
}

#[derive(Serialize)]
pub struct BlurRegion {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pii_type: String,
}

#[derive(Serialize)]
pub struct LlmResponse {
    pub response: String,
    pub model: String,
    pub tokens_used: u32,
}

#[tauri::command]
pub fn run_ocr(
    app: tauri::AppHandle,
    image_id: String,
    lang: Option<String>,
    store: tauri::State<'_, ImageStore>,
) -> Result<OcrResult, String> {
    // Parse UUID
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {}", e))?;

    // Get image from store
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    // Convert to RGB8 format (tesseract expects raw pixel data)
    let rgb_image = image.to_rgb8();
    let width = rgb_image.width();
    let height = rgb_image.height();
    let raw_data = rgb_image.into_raw();

    let lang_str = lang.unwrap_or_else(|| "eng".to_string());

    // Convert u32 dimensions to i32 for Tesseract API
    let width_i32 = width as i32;
    let height_i32 = height as i32;
    let bytes_per_line = (width * 3) as i32;

    // Determine tessdata path.
    // - Flatpak: tessdata is installed at /app/share/tessdata/
    // - Bundled app: tessdata is in the app resource directory
    let bundled_path: PathBuf;
    let tessdata_path: &str = if std::env::var("FLATPAK_ID").is_ok() {
        "/app/share/tessdata"
    } else {
        use tauri::Manager;
        bundled_path = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?
            .join("resources")
            .join("tessdata");
        bundled_path
            .to_str()
            .ok_or_else(|| "tessdata path contains invalid UTF-8".to_string())?
    };

    let mut tess = Tesseract::new(Some(tessdata_path), Some(&lang_str))
        .map_err(|e| format!("Failed to initialize Tesseract with language '{}': {}", lang_str, e))?
        .set_frame(&raw_data, width_i32, height_i32, 3, bytes_per_line)
        .map_err(|e| format!("Failed to set image: {}", e))?
        .recognize()
        .map_err(|e| format!("OCR recognition failed: {}", e))?;

    let text = tess
        .get_text()
        .map_err(|e| format!("OCR get_text failed: {}", e))?;

    // Parse TSV output for per-word bounding boxes.
    // TSV columns: level page_num block_num par_num line_num word_num left top width height conf text
    // Level 5 = word. Skip entries with conf < 0 (non-word rows) or empty text.
    let tsv = tess
        .get_tsv_text(0)
        .map_err(|e| format!("OCR get_tsv_text failed: {}", e))?;

    let regions: Vec<OcrRegion> = tsv
        .lines()
        .skip(1) // skip header
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
            let word_text = cols[11].trim().to_string();
            if word_text.is_empty() {
                return None;
            }
            let x: u32 = cols[6].parse().ok()?;
            let y: u32 = cols[7].parse().ok()?;
            let w: u32 = cols[8].parse().ok()?;
            let h: u32 = cols[9].parse().ok()?;
            Some(OcrRegion {
                text: word_text,
                x,
                y,
                w,
                h,
                confidence: conf,
            })
        })
        .collect();

    Ok(OcrResult { text, regions })
}

#[tauri::command]
pub fn auto_blur_pii(
    app: tauri::AppHandle,
    image_id: String,
    store: tauri::State<'_, ImageStore>,
) -> Result<Vec<BlurRegion>, String> {
    use tauri::Manager;
    // Parse UUID
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {}", e))?;

    // Get image from store
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    // Convert to RGB8 for Tesseract
    let rgb_image = image.to_rgb8();
    let width = rgb_image.width();
    let height = rgb_image.height();
    let raw_data = rgb_image.into_raw();

    let width_i32 = width as i32;
    let height_i32 = height as i32;
    let bytes_per_line = (width * 3) as i32;

    // Resolve tessdata path
    let bundled_path: PathBuf;
    let tessdata_path: &str = if std::env::var("FLATPAK_ID").is_ok() {
        "/app/share/tessdata"
    } else {
        bundled_path = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?
            .join("resources")
            .join("tessdata");
        bundled_path
            .to_str()
            .ok_or_else(|| "tessdata path contains invalid UTF-8".to_string())?
    };

    // Run OCR with bounding boxes
    let mut tess = Tesseract::new(Some(tessdata_path), Some("eng"))
        .map_err(|e| format!("Failed to initialize Tesseract: {}", e))?
        .set_frame(&raw_data, width_i32, height_i32, 3, bytes_per_line)
        .map_err(|e| format!("Failed to set image: {}", e))?
        .recognize()
        .map_err(|e| format!("OCR recognition failed: {}", e))?;

    let tsv = tess
        .get_tsv_text(0)
        .map_err(|e| format!("OCR get_tsv_text failed: {}", e))?;

    // Parse TSV into ai::ocr::OcrRegion for PII detection
    let ocr_regions: Vec<crate::ai::ocr::OcrRegion> = tsv
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
            let word_text = cols[11].trim().to_string();
            if word_text.is_empty() {
                return None;
            }
            Some(crate::ai::ocr::OcrRegion {
                text: word_text,
                x: cols[6].parse().ok()?,
                y: cols[7].parse().ok()?,
                w: cols[8].parse().ok()?,
                h: cols[9].parse().ok()?,
                confidence: conf,
            })
        })
        .collect();

    // Run PII detection
    let pii_matches = crate::ai::pii::detect_pii(&ocr_regions)
        .map_err(|e| format!("PII detection failed: {}", e))?;

    // Convert PiiMatch → BlurRegion
    let blur_regions = pii_matches
        .into_iter()
        .map(|m| BlurRegion {
            x: m.x,
            y: m.y,
            w: m.w,
            h: m.h,
            pii_type: m.pii_type,
        })
        .collect();

    Ok(blur_regions)
}

#[tauri::command]
pub fn analyze_llm(
    _image_id: String,
    _prompt: Option<String>,
    _provider: String,
) -> Result<LlmResponse, String> {
    // TODO: implement LLM vision analysis
    Err("Not yet implemented".into())
}
