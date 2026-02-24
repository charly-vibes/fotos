use crate::capture::ImageStore;
use base64::prelude::*;
use serde::Serialize;
use std::io::Cursor;
use std::sync::Arc;
use tauri::Emitter;
use uuid::Uuid;

#[derive(Serialize, Clone)]
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
    let _ = monitor;
    tracing::info!("take_screenshot: mode={mode}");

    let image = match mode.as_str() {
        "fullscreen" => {
            // Inside a Flatpak sandbox, direct capture APIs are blocked.
            // Route through the XDG Desktop Portal instead.
            let in_flatpak = std::env::var("FLATPAK_ID").is_ok();
            tracing::info!("take_screenshot: in_flatpak={in_flatpak}");
            if in_flatpak {
                tracing::info!("take_screenshot: routing to portal backend");
                crate::capture::portal::capture_via_portal()
                    .await
                    .map_err(|e| {
                        tracing::error!("take_screenshot: portal failed: {e}");
                        format!("Portal capture failed: {}", e)
                    })?
            } else {
                tracing::info!("take_screenshot: routing to xcap backend");
                crate::capture::xcap_backend::capture_fullscreen()
                    .await
                    .map_err(|e| {
                        tracing::error!("take_screenshot: xcap failed: {e}");
                        format!("Capture failed: {}", e)
                    })?
            }
        }
        "region" => {
            return Err(
                "Region capture is handled in-app; use fullscreen + crop_image".into(),
            );
        }
        other => {
            return Err(format!(
                "Mode '{}' not yet implemented",
                other
            ));
        }
    };
    tracing::info!("take_screenshot: image captured ({}x{})", image.width(), image.height());

    let image = Arc::new(image);
    let width = image.width();
    let height = image.height();

    // Generate UUID and store image (Arc clone, no pixel copy)
    let id = Uuid::new_v4();
    store.insert(id, Arc::clone(&image));

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
pub fn crop_image(
    image_id: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    store: tauri::State<'_, ImageStore>,
) -> Result<ScreenshotResponse, String> {
    let id = Uuid::parse_str(&image_id)
        .map_err(|_| format!("Invalid image ID: {image_id}"))?;
    let base = store
        .get(&id)
        .ok_or_else(|| format!("No image found for ID: {image_id}"))?;

    // Clamp to image bounds to avoid panic
    let img_w = base.width();
    let img_h = base.height();
    let x = x.min(img_w.saturating_sub(1));
    let y = y.min(img_h.saturating_sub(1));
    let width = width.min(img_w - x);
    let height = height.min(img_h - y);

    let cropped = Arc::new(base.crop_imm(x, y, width, height));
    let new_id = Uuid::new_v4();
    store.insert(new_id, Arc::clone(&cropped));

    let mut png_data = Vec::new();
    cropped
        .write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| format!("PNG encoding failed: {e}"))?;
    let data_url = format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(&png_data)
    );

    Ok(ScreenshotResponse {
        id: new_id.to_string(),
        width,
        height,
        data_url,
    })
}

#[tauri::command]
pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    // TODO: enumerate monitors
    Err("Not yet implemented".into())
}

#[tauri::command]
pub fn list_windows() -> Result<Vec<WindowInfo>, String> {
    // TODO: enumerate windows
    Err("Not yet implemented".into())
}
