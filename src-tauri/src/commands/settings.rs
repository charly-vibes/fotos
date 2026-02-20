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

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            default_mode: "fullscreen".to_string(),
            save_directory: "~/Pictures/Fotos".to_string(),
            default_format: "png".to_string(),
            jpeg_quality: 90,
            copy_to_clipboard_after_capture: true,
            include_mouse_cursor: false,
            delay_ms: 0,
        }
    }
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

impl Default for AnnotationSettings {
    fn default() -> Self {
        Self {
            default_stroke_color: "#FF0000".to_string(),
            default_stroke_width: 2.0,
            default_font_size: 16.0,
            default_font_family: "sans-serif".to_string(),
            step_number_color: "#FF0000".to_string(),
            step_number_size: 24.0,
            blur_radius: 10.0,
        }
    }
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

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            ocr_language: "eng".to_string(),
            default_llm_provider: "claude".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3.2-vision".to_string(),
            claude_model: "claude-sonnet-4-5".to_string(),
            openai_model: "gpt-4o".to_string(),
            gemini_model: "gemini-2.0-flash-exp".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub show_ai_panel: bool,
    pub show_status_bar: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            show_ai_panel: true,
            show_status_bar: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub capture: CaptureSettings,
    pub annotation: AnnotationSettings,
    pub ai: AiSettings,
    pub ui: UiSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            capture: CaptureSettings::default(),
            annotation: AnnotationSettings::default(),
            ai: AiSettings::default(),
            ui: UiSettings::default(),
        }
    }
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    // Tracer-bullet: return hardcoded defaults (no persistence)
    Ok(Settings::default())
}

#[tauri::command]
pub async fn set_settings(_settings: Settings) -> Result<(), String> {
    // Tracer-bullet: no-op (no persistence)
    Ok(())
}

#[tauri::command]
pub async fn set_api_key(_provider: String, _key: String) -> Result<(), String> {
    // Tracer-bullet: no-op (no keychain integration)
    Ok(())
}
