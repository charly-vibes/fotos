pub mod ai;
pub mod capture;
pub mod files;
pub mod settings;

/// Tracer-bullet: verify Tauri IPC round-trip
#[tauri::command]
pub async fn ping() -> Result<String, String> {
    Ok("pong".to_string())
}
