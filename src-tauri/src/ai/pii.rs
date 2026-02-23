/// PII (Personally Identifiable Information) detection.
///
/// Runs OCR first, then applies regex pattern matching on extracted text
/// with bounding boxes to identify sensitive information.
use anyhow::Result;

pub struct PiiMatch {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pii_type: String,
}

pub fn detect_pii(_ocr_regions: &[super::ocr::OcrRegion]) -> Result<Vec<PiiMatch>> {
    // TODO: match OCR text against PII patterns:
    // - Email addresses
    // - Phone numbers (US format)
    // - SSN
    // - Credit card numbers
    // - API keys (sk-*, AKIA*, ghp_*)
    // - IP addresses
    // - AWS ARNs
    anyhow::bail!("PII detection not yet implemented")
}
