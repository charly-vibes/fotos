use serde::Serialize;

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
) -> Result<OcrResult, String> {
    // TODO: implement Tesseract OCR
    Err("Not yet implemented".into())
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
