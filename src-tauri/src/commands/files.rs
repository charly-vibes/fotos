use crate::capture::ImageStore;
use chrono::Local;
use directories::UserDirs;
use image::Rgba;
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Deserialize)]
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
}

#[tauri::command]
pub async fn save_image(
    image_id: String,
    annotations: Vec<Annotation>,
    _format: String,
    path: String,
    store: tauri::State<'_, ImageStore>,
) -> Result<String, String> {
    // Parse UUID
    let uuid = Uuid::parse_str(&image_id)
        .map_err(|e| format!("Invalid image ID: {}", e))?;

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
        expand_tilde(&path)
    };

    // Create parent directory if needed
    if let Some(parent) = save_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Save as PNG
    composite
        .save(&save_path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(save_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn copy_to_clipboard(
    image_id: String,
    annotations: Vec<Annotation>,
) -> Result<(), String> {
    // TODO: composite and copy to clipboard
    Err("Not yet implemented".into())
}

#[tauri::command]
pub async fn export_annotations(
    image_id: String,
    annotations: Vec<Annotation>,
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

    // Skip invalid rectangles
    if width <= 0 || height <= 0 {
        return;
    }

    let stroke_color = parse_color(&anno.stroke_color.clone().unwrap_or_else(|| "#FF0000".to_string()));
    let stroke_width = anno.stroke_width.unwrap_or(2.0) as i32;

    // Draw multiple rects to simulate stroke width
    for i in 0..stroke_width {
        let offset_x = x - i;
        let offset_y = y - i;
        let rect_width = (width + i * 2).max(1) as u32;
        let rect_height = (height + i * 2).max(1) as u32;

        // Ensure rectangle is within image bounds
        if offset_x >= 0 && offset_y >= 0 {
            let rect = Rect::at(offset_x, offset_y).of_size(rect_width, rect_height);
            draw_hollow_rect_mut(composite, rect, stroke_color);
        }
    }
}

fn parse_color(hex: &str) -> Rgba<u8> {
    // Parse #RRGGBB format
    let hex = hex.trim_start_matches('#');

    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Rgba([r, g, b, 255])
    } else {
        // Default to red if invalid
        Rgba([255, 0, 0, 255])
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

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = UserDirs::new().and_then(|dirs| dirs.home_dir().to_path_buf().into()) {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}
