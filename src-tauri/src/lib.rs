pub mod ai;
pub mod capture;
pub mod commands;
pub mod credentials;
pub mod ipc;
#[cfg(target_os = "linux")]
mod dbus;

use base64::prelude::*;
use std::io::Cursor;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use uuid::Uuid;

fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

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
        tracing::debug!("do_capture_and_emit: capture already in flight, ignoring");
        return;
    }

    tracing::info!("do_capture_and_emit: starting capture for event '{event_name}'");

    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => {
            tracing::error!("do_capture_and_emit: main window not found");
            is_capturing.store(false, Ordering::SeqCst);
            return;
        }
    };

    // In Flatpak the portal captures the screen including our window, so hide
    // first regardless of backend so the app doesn't appear in the shot.
    let _ = window.hide();
    // On X11 ~150ms is enough; on Wayland the compositor needs a full frame.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let image_store = app.state::<capture::ImageStore>();
    let result: Result<commands::capture::ScreenshotResponse, String> = async {
        #[cfg(target_os = "linux")]
        let in_flatpak = std::env::var("FLATPAK_ID").is_ok();
        #[cfg(target_os = "linux")]
        tracing::info!("do_capture_and_emit: in_flatpak={in_flatpak}");

        #[cfg(target_os = "linux")]
        let image = if in_flatpak {
            tracing::info!("do_capture_and_emit: using portal backend");
            capture::portal::capture_via_portal()
                .await
                .map_err(|e| e.to_string())?
        } else {
            tracing::info!("do_capture_and_emit: using xcap backend");
            capture::xcap_backend::capture_fullscreen()
                .await
                .map_err(|e| e.to_string())?
        };
        #[cfg(not(target_os = "linux"))]
        let image = {
            tracing::info!("do_capture_and_emit: using xcap backend");
            capture::xcap_backend::capture_fullscreen()
                .await
                .map_err(|e| e.to_string())?
        };
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
        Ok(ref payload) => {
            tracing::info!(
                "do_capture_and_emit: captured {}x{}, emitting '{event_name}'",
                payload.width,
                payload.height
            );
            let _ = app.emit(event_name, payload);
        }
        Err(ref e) => {
            tracing::error!("do_capture_and_emit: capture failed: {e}");
            let _ = app.emit(event_name, serde_json::json!({ "error": e }));
        }
    }
}

pub fn run() {
    init_logging();
    tracing::info!(
        "Fotos starting (FLATPAK_ID={:?})",
        std::env::var("FLATPAK_ID").ok()
    );

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

            // Start the IPC server so fotos-mcp can connect.
            let ipc_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = ipc::server::start_ipc_server(ipc_handle).await {
                    tracing::error!("IPC server exited: {e}");
                }
            });

            // Start the D-Bus service for GNOME Shell integration (Linux only).
            #[cfg(target_os = "linux")]
            {
                let dbus_handle = handle.clone();
                let dbus_capturing = is_capturing.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = dbus::start_service(dbus_handle, dbus_capturing).await {
                        tracing::warn!("D-Bus service failed to start: {e}");
                    }
                });
            }

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
            commands::files::import_annotations,
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::settings::set_api_key,
            commands::settings::get_api_key,
            commands::settings::delete_api_key,
            commands::settings::test_api_key,
        ])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let _ = std::fs::remove_file(ipc::server::socket_path());
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Fotos");
}
