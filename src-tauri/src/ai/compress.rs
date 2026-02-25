/// Image compression utility for LLM API submission.
///
/// Resizes images that exceed a maximum dimension and encodes as JPEG
/// to reduce API costs and stay within provider size limits.
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use image::DynamicImage;

/// Resize and JPEG-encode an image for LLM submission.
///
/// If the image's longest side exceeds `max_dim`, it is resized proportionally.
/// The result is JPEG-encoded at `quality` (1–100) and base64-encoded.
pub fn compress_for_llm(image: &DynamicImage, max_dim: u32, quality: u8) -> Result<String> {
    let (w, h) = (image.width(), image.height());

    let resized = if w > max_dim || h > max_dim {
        let scale = max_dim as f32 / w.max(h) as f32;
        let new_w = ((w as f32 * scale).round() as u32).max(1);
        let new_h = ((h as f32 * scale).round() as u32).max(1);
        image.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3)
    } else {
        image.clone()
    };

    let rgb = resized.to_rgb8();
    let mut buf = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    encoder.encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        image::ExtendedColorType::Rgb8,
    )?;

    Ok(STANDARD.encode(&buf))
}
