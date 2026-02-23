pub mod ai;
pub mod capture;
pub mod commands;
pub mod credentials;
pub mod ipc;

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
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::capture::take_screenshot,
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
