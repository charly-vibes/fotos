use crate::capture::ImageStore;
use serde::Serialize;
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

    // Determine tessdata path. Inside a Flatpak, traineddata files are installed
    // at /app/share/tessdata/ — pass that explicitly. Outside Flatpak, pass None
    // so Tesseract falls back to TESSDATA_PREFIX or its built-in search paths.
    let tessdata_path: Option<&str> = if std::env::var("FLATPAK_ID").is_ok() {
        Some("/app/share/tessdata")
    } else {
        None
    };

    let text = Tesseract::new(tessdata_path, Some(&lang_str))
        .map_err(|e| format!("Failed to initialize Tesseract with language '{}': {}", lang_str, e))?
        .set_frame(&raw_data, width_i32, height_i32, 3, bytes_per_line)
        .map_err(|e| format!("Failed to set image: {}", e))?
        .get_text()
        .map_err(|e| format!("OCR failed: {}", e))?;

    // For tracer-bullet: return empty regions vec (skip per-word bounding boxes)
    Ok(OcrResult {
        text,
        regions: vec![],
    })
}

#[tauri::command]
pub fn auto_blur_pii(_image_id: String) -> Result<Vec<BlurRegion>, String> {
    // TODO: implement PII detection
    Err("Not yet implemented".into())
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
