use serde::Serialize;

#[derive(Serialize)]
pub struct ScreenshotResponse {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub data_url: String,
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
) -> Result<ScreenshotResponse, String> {
    // TODO: implement capture via platform-specific backend
    Err("Not yet implemented".into())
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
