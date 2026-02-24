pub mod ai;
pub mod capture;
pub mod commands;
pub mod credentials;
pub mod ipc;

use base64::prelude::*;
use std::io::Cursor;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use uuid::Uuid;

async fn do_capture_and_emit(
    app: &tauri::AppHandle,
    event_name: &'static str,
    is_capturing: Arc<AtomicBool>,
) {
    // Guard: ignore if a capture is already in flight.
    if is_capturing
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => {
            is_capturing.store(false, Ordering::SeqCst);
            return;
        }
    };

    let _ = window.hide();
    // On X11 ~150ms is enough; on Wayland the compositor needs a full frame.
    // 300ms is conservative but reliable on both. Future: use portal.rs on Wayland.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let image_store = app.state::<capture::ImageStore>();
    let result: Result<commands::capture::ScreenshotResponse, String> = async {
        let image = capture::xcap_backend::capture_fullscreen().await.map_err(|e| e.to_string())?;
        let image = Arc::new(image);
        let id = Uuid::new_v4();
        image_store.insert(id, Arc::clone(&image));
        let mut png = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        Ok(commands::capture::ScreenshotResponse {
            id: id.to_string(),
            width: image.width(),
            height: image.height(),
            data_url: format!("data:image/png;base64,{}", BASE64_STANDARD.encode(&png)),
        })
    }
    .await;

    let _ = window.show();
    let _ = window.set_focus();
    is_capturing.store(false, Ordering::SeqCst);

    match result {
        Ok(payload) => {
            let _ = app.emit(event_name, payload);
        }
        Err(e) => {
            let _ = app.emit(event_name, serde_json::json!({ "error": e }));
        }
    }
}

pub fn run() {
    let image_store = capture::ImageStore::new();

    tauri::Builder::default()
        .manage(image_store)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let is_capturing = Arc::new(AtomicBool::new(false));

            let r1 = app.global_shortcut().on_shortcut("ctrl+shift+s", {
                let handle = handle.clone();
                let is_capturing = is_capturing.clone();
                move |_app, _shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let handle = handle.clone();
                    let is_capturing = is_capturing.clone();
                    tauri::async_runtime::spawn(async move {
                        do_capture_and_emit(&handle, "global-capture-region", is_capturing).await;
                    });
                }
            });
            if let Err(e) = r1 {
                eprintln!("Warning: could not register Ctrl+Shift+S global shortcut: {e}");
            }

            let r2 = app.global_shortcut().on_shortcut("ctrl+shift+a", {
                let handle = handle.clone();
                let is_capturing = is_capturing.clone();
                move |_app, _shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let handle = handle.clone();
                    let is_capturing = is_capturing.clone();
                    tauri::async_runtime::spawn(async move {
                        do_capture_and_emit(&handle, "global-capture-fullscreen", is_capturing)
                            .await;
                    });
                }
            });
            if let Err(e) = r2 {
                eprintln!("Warning: could not register Ctrl+Shift+A global shortcut: {e}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::capture::take_screenshot,
            commands::capture::crop_image,
            commands::capture::list_monitors,
            commands::capture::list_windows,
            commands::ai::run_ocr,
            commands::ai::auto_blur_pii,
            commands::ai::analyze_llm,
            commands::files::save_image,
            commands::files::composite_image,
            commands::files::copy_to_clipboard,
            commands::files::export_annotations,
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::settings::set_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Fotos");
}
