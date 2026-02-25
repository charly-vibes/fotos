use crate::capture::ImageStore;
use base64::prelude::*;
use serde::Serialize;
use std::io::Cursor;
use std::sync::Arc;
use tauri::Emitter;
use uuid::Uuid;
use xcap::{Monitor, Window};

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
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[derive(Serialize)]
pub struct WindowInfo {
    pub id: u32,
    pub title: String,
    pub app_name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub async fn take_screenshot(
    mode: String,
    monitor: Option<u32>,
    window_id: Option<u32>,
    store: tauri::State<'_, ImageStore>,
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<ScreenshotResponse, String> {
    tracing::info!("take_screenshot: mode={mode}");

    let image = match mode.as_str() {
        "fullscreen" => {
            // Hide the app window so it doesn't appear in the screenshot.
            tracing::info!("take_screenshot: hiding window");
            let _ = window.hide();
            // Give the compositor a moment to actually hide the window.
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            let in_flatpak = std::env::var("FLATPAK_ID").is_ok();
            tracing::info!("take_screenshot: in_flatpak={in_flatpak}");
            let result = if in_flatpak {
                tracing::info!("take_screenshot: routing to portal backend");
                crate::capture::portal::capture_via_portal()
                    .await
                    .map_err(|e| {
                        tracing::error!("take_screenshot: portal failed: {e}");
                        format!("Portal capture failed: {}", e)
                    })
            } else {
                tracing::info!("take_screenshot: routing to xcap backend");
                crate::capture::xcap_backend::capture_fullscreen()
                    .await
                    .map_err(|e| {
                        tracing::error!("take_screenshot: xcap failed: {e}");
                        format!("Capture failed: {}", e)
                    })
            };

            // Always restore the window before returning.
            tracing::info!("take_screenshot: restoring window");
            let _ = window.show();
            let _ = window.set_focus();

            result?
        }
        "monitor" => {
            let index = monitor.ok_or("monitor index required for mode 'monitor'")?;
            tracing::info!("take_screenshot: monitor index={index}");

            let _ = window.hide();
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            let result = crate::capture::xcap_backend::capture_monitor(index)
                .await
                .map_err(|e| {
                    tracing::error!("take_screenshot: monitor capture failed: {e}");
                    format!("Monitor capture failed: {}", e)
                });

            let _ = window.show();
            let _ = window.set_focus();

            result?
        }
        "window" => {
            let wid = window_id.ok_or("window_id required for mode 'window'")?;
            tracing::info!("take_screenshot: window_id={wid}");

            crate::capture::xcap_backend::capture_window(wid)
                .await
                .map_err(|e| {
                    tracing::error!("take_screenshot: window capture failed: {e}");
                    format!("Window capture failed: {}", e)
                })?
        }
        "region" => {
            return Err(
                "Region capture is handled in-app; use fullscreen + crop_image".into(),
            );
        }
        other => {
            return Err(format!("Unknown capture mode '{}'", other));
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
pub async fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    tokio::task::spawn_blocking(|| {
        let monitors =
            Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;
        monitors
            .into_iter()
            .map(|m| {
                Ok(MonitorInfo {
                    id: m.id().map_err(|e| format!("monitor.id: {e}"))?,
                    name: m.name().map_err(|e| format!("monitor.name: {e}"))?,
                    x: m.x().map_err(|e| format!("monitor.x: {e}"))?,
                    y: m.y().map_err(|e| format!("monitor.y: {e}"))?,
                    width: m.width().map_err(|e| format!("monitor.width: {e}"))?,
                    height: m.height().map_err(|e| format!("monitor.height: {e}"))?,
                    is_primary: m.is_primary().map_err(|e| format!("monitor.is_primary: {e}"))?,
                })
            })
            .collect()
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn list_windows() -> Result<Vec<WindowInfo>, String> {
    tokio::task::spawn_blocking(|| {
        let windows =
            Window::all().map_err(|e| format!("Failed to enumerate windows: {e}"))?;
        windows
            .into_iter()
            .map(|w| {
                Ok(WindowInfo {
                    id: w.id().map_err(|e| format!("window.id: {e}"))?,
                    title: w.title().map_err(|e| format!("window.title: {e}"))?,
                    app_name: w.app_name().map_err(|e| format!("window.app_name: {e}"))?,
                    x: w.x().map_err(|e| format!("window.x: {e}"))?,
                    y: w.y().map_err(|e| format!("window.y: {e}"))?,
                    width: w.width().map_err(|e| format!("window.width: {e}"))?,
                    height: w.height().map_err(|e| format!("window.height: {e}"))?,
                })
            })
            .collect()
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
