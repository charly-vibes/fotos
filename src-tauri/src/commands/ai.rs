use crate::capture::ImageStore;
use serde::Serialize;
use tesseract_rs::TesseractAPI;
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
pub async fn run_ocr(
    image_id: String,
    lang: Option<String>,
    store: tauri::State<'_, ImageStore>,
) -> Result<OcrResult, String> {
    // Parse UUID
    let uuid = Uuid::parse_str(&image_id)
        .map_err(|e| format!("Invalid image ID: {}", e))?;

    // Get image from store
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    // Convert to RGB8 format (tesseract expects raw pixel data)
    let rgb_image = image.to_rgb8();
    let width = rgb_image.width();
    let height = rgb_image.height();
    let raw_data = rgb_image.into_raw();

    // Initialize Tesseract
    let lang_str = lang.unwrap_or_else(|| "eng".to_string());
    let api = TesseractAPI::new();

    // Initialize with default tessdata directory (empty string = use system default)
    // On Linux, this is typically /usr/share/tessdata or /usr/share/tesseract-ocr/*/tessdata
    api.init("", &lang_str)
        .map_err(|e| format!("Failed to initialize Tesseract with language '{}': {}", lang_str, e))?;

    // Convert u32 dimensions to i32 for Tesseract API
    let width_i32 = width as i32;
    let height_i32 = height as i32;
    let bytes_per_line = (width * 3) as i32;

    api.set_image(&raw_data, width_i32, height_i32, 3, bytes_per_line)
        .map_err(|e| format!("Failed to set image: {}", e))?;

    // Run OCR
    let text = api
        .get_utf8_text()
        .map_err(|e| format!("OCR failed: {}", e))?;

    // For tracer-bullet: return empty regions vec (skip per-word bounding boxes)
    Ok(OcrResult {
        text,
        regions: vec![],
    })
}

#[tauri::command]
pub async fn auto_blur_pii(image_id: String) -> Result<Vec<BlurRegion>, String> {
    // TODO: implement PII detection
    Err("Not yet implemented".into())
}

#[tauri::command]
pub async fn analyze_llm(
    image_id: String,
    prompt: Option<String>,
    provider: String,
) -> Result<LlmResponse, String> {
    // TODO: implement LLM vision analysis
    Err("Not yet implemented".into())
}
