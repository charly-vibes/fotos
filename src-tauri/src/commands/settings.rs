use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CaptureSettings {
    pub default_mode: String,
    pub include_mouse_cursor: bool,
    pub delay_ms: u32,
    pub save_directory: String,
    pub default_format: String,
    pub jpeg_quality: u8,
    pub copy_to_clipboard_after_capture: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AnnotationSettings {
    pub default_stroke_color: String,
    pub default_stroke_width: f64,
    pub default_font_size: f64,
    pub default_font_family: String,
    pub step_number_color: String,
    pub step_number_size: f64,
    pub blur_radius: f64,
}

#[derive(Serialize, Deserialize)]
pub struct AiSettings {
    pub ocr_language: String,
    pub default_llm_provider: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub claude_model: String,
    pub openai_model: String,
    pub gemini_model: String,
}

#[derive(Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub show_ai_panel: bool,
    pub show_status_bar: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub capture: CaptureSettings,
    pub annotation: AnnotationSettings,
    pub ai: AiSettings,
    pub ui: UiSettings,
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    // TODO: load settings from tauri-plugin-store
    Err("Not yet implemented".into())
}

#[tauri::command]
pub async fn set_settings(settings: Settings) -> Result<(), String> {
    // TODO: save settings via tauri-plugin-store
    Err("Not yet implemented".into())
}

#[tauri::command]
pub async fn set_api_key(provider: String, key: String) -> Result<(), String> {
    // TODO: store API key in OS keychain via keyring crate
    Err("Not yet implemented".into())
}
