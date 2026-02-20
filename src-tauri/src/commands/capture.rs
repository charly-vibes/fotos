use crate::capture::ImageStore;
use base64::prelude::*;
use serde::Serialize;
use std::io::Cursor;
use tauri::Emitter;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ScreenshotResponse {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub data_url: String,
}

#[derive(Serialize, Clone)]
struct ScreenshotReadyEvent {
    id: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
pub struct MonitorInfo {
    pub index: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[derive(Serialize)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub app_name: String,
}

#[tauri::command]
pub async fn take_screenshot(
    mode: String,
    monitor: Option<u32>,
    store: tauri::State<'_, ImageStore>,
    app: tauri::AppHandle,
) -> Result<ScreenshotResponse, String> {
    // Tracer-bullet: only support fullscreen mode
    if mode != "fullscreen" {
        return Err(format!("Mode '{}' not yet implemented (tracer supports fullscreen only)", mode));
    }

    // Capture fullscreen using xcap
    let image = crate::capture::xcap_backend::capture_fullscreen()
        .await
        .map_err(|e| format!("Capture failed: {}", e))?;

    let width = image.width();
    let height = image.height();

    // Generate UUID and store image
    let id = Uuid::new_v4();
    store.insert(id, image.clone());

    // Convert to base64 PNG data URL
    let mut png_data = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| format!("PNG encoding failed: {}", e))?;

    let base64_data = BASE64_STANDARD.encode(&png_data);
    let data_url = format!("data:image/png;base64,{}", base64_data);

    // Emit screenshot-ready event
    let event = ScreenshotReadyEvent {
        id: id.to_string(),
        width,
        height,
    };
    app.emit("screenshot-ready", event)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(ScreenshotResponse {
        id: id.to_string(),
        width,
        height,
        data_url,
    })
}

#[tauri::command]
pub async fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    // TODO: enumerate monitors
    Err("Not yet implemented".into())
}

#[tauri::command]
pub async fn list_windows() -> Result<Vec<WindowInfo>, String> {
    // TODO: enumerate windows
    Err("Not yet implemented".into())
}
