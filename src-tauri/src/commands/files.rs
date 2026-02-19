use serde::{Deserialize, Serialize};

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
    format: String,
    path: String,
) -> Result<String, String> {
    // TODO: composite and save image
    Err("Not yet implemented".into())
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
