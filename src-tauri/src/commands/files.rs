use crate::capture::ImageStore;
use base64::Engine;
use chrono::Local;
use directories::UserDirs;
use image::Rgba;
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use serde::Deserialize;
use std::io::Cursor;
use std::path::PathBuf;
use uuid::Uuid;

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
    pub points: Option<Vec<serde_json::Value>>,
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
    // Parse UUID
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {}", e))?;

    // Look up image in store
    let base_image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    // Clone to RGBA for compositing
    let mut composite = base_image.to_rgba8();

    // Composite annotations
    for anno in annotations {
        if anno.annotation_type == "rect" {
            composite_rectangle(&mut composite, &anno);
        }
        // Skip other annotation types for tracer-bullet
    }

    // Determine save path
    let save_path = if path.is_empty() {
        generate_default_path()?
    } else {
        expand_tilde(&path)?
    };

    // Reject path traversal: ensure the resolved path stays within the user's home directory.
    let home = UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .ok_or("Could not determine home directory for path validation")?;

    // Create parent directory first so canonicalize can resolve the full path.
    if let Some(parent) = save_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let canonical = save_path
        .canonicalize()
        .unwrap_or_else(|_| save_path.clone());
    if !canonical.starts_with(&home) {
        return Err(format!(
            "Save path '{}' is outside the home directory",
            save_path.display()
        ));
    }

    // Save as PNG
    composite
        .save(&save_path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(save_path.to_string_lossy().to_string())
}

/// Composite annotations onto an image and return the result as a base64-encoded PNG.
/// Used by the frontend for export preview and clipboard operations.
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

    for anno in annotations {
        if anno.annotation_type == "rect" {
            composite_rectangle(&mut composite, &anno);
        }
    }

    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(composite)
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(buf.into_inner()))
}

#[tauri::command]
pub fn copy_to_clipboard(_image_id: String, _annotations: Vec<Annotation>) -> Result<(), String> {
    // TODO: composite and copy to clipboard
    Err("Not yet implemented".into())
}

#[tauri::command]
pub fn export_annotations(
    _image_id: String,
    _annotations: Vec<Annotation>,
) -> Result<String, String> {
    // TODO: export annotations as JSON
    Err("Not yet implemented".into())
}

// Helper functions

fn composite_rectangle(composite: &mut image::RgbaImage, anno: &Annotation) {
    let x = anno.x as i32;
    let y = anno.y as i32;
    let width = anno.width.unwrap_or(0.0) as i32;
    let height = anno.height.unwrap_or(0.0) as i32;

    if width <= 0 || height <= 0 {
        return;
    }

    // Apply annotation opacity to fill alpha.
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

    // Stroke: centered on the rect edge (half inside, half outside).
    let stroke_color_str = anno
        .stroke_color
        .as_deref()
        .unwrap_or("#FF0000");
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
        x: f64, y: f64, w: f64, h: f64,
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
            points: None,
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

    /// A 100×100 solid white image with a red stroke rect at (10,10) 30×20.
    /// The pixel at the top-left corner of the rect should be red.
    #[test]
    fn compositing_stroke_pixel_is_stroke_color() {
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let anno = make_annotation("rect", 10.0, 10.0, 30.0, 20.0, Some("#ff0000"), Some("transparent"), Some(1.0), Some(1.0));
        composite_rectangle(&mut img, &anno);
        // Top-left corner of rect should be red (stroke).
        assert_eq!(*img.get_pixel(10, 10), Rgba([255, 0, 0, 255]));
    }

    /// Pixel outside the rect should be unchanged (white).
    #[test]
    fn compositing_outside_pixel_unchanged() {
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let anno = make_annotation("rect", 10.0, 10.0, 30.0, 20.0, Some("#ff0000"), Some("transparent"), Some(1.0), Some(1.0));
        composite_rectangle(&mut img, &anno);
        assert_eq!(*img.get_pixel(0, 0), Rgba([255, 255, 255, 255]));
        assert_eq!(*img.get_pixel(99, 99), Rgba([255, 255, 255, 255]));
    }

    /// Fill color should appear inside the rect.
    #[test]
    fn compositing_fill_pixel_is_fill_color() {
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let anno = make_annotation("rect", 10.0, 10.0, 30.0, 20.0, Some("transparent"), Some("#0000ff"), Some(0.0), Some(1.0));
        composite_rectangle(&mut img, &anno);
        // Interior pixel (not on edge) should be blue.
        let px = img.get_pixel(20, 15);
        assert_eq!(px[2], 255); // blue channel
        assert_eq!(px[0], 0);   // not red
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
}
