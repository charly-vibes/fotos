/// Tesseract OCR wrapper.
///
/// Extracts text from screenshot images with bounding box information
/// for each detected word/region.

use anyhow::Result;

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

pub fn run_ocr(_image: &image::DynamicImage, _lang: &str) -> Result<OcrOutput> {
    // TODO: use tesseract-rs to extract text
    // Default language: "eng", PSM mode 3
    anyhow::bail!("OCR not yet implemented")
}
