use reqwest::header;
use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "prefs.json";
const SCHEMA_VERSION: u64 = 2;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
            default_mode: "region".to_string(),
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
#[serde(rename_all = "camelCase")]
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

/// A user-defined OpenAI-compatible LLM endpoint.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LlmEndpoint {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
}

fn default_endpoints() -> Vec<LlmEndpoint> {
    vec![
        LlmEndpoint {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o".to_string(),
        },
        LlmEndpoint {
            id: "ollama-local".to_string(),
            name: "Ollama (local)".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            model: "llava:7b".to_string(),
        },
    ]
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub ocr_language: String,
    pub default_llm_provider: String,
    /// User-defined OpenAI-compatible endpoints (replaces fixed openai/ollama fields).
    pub endpoints: Vec<LlmEndpoint>,
    pub claude_model: String,
    pub gemini_model: String,
    pub image_max_dim: u32,
    pub image_quality: u8,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            ocr_language: "eng".to_string(),
            default_llm_provider: "claude".to_string(),
            endpoints: default_endpoints(),
            claude_model: "claude-sonnet-4-20250514".to_string(),
            gemini_model: "gemini-2.0-flash".to_string(),
            image_max_dim: 2048,
            image_quality: 85,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    pub theme: String,
    pub show_ai_panel: bool,
    pub show_status_bar: bool,
    pub smooth_zoom: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            show_ai_panel: true,
            show_status_bar: true,
            smooth_zoom: true,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    pub capture: CaptureSettings,
    pub annotation: AnnotationSettings,
    pub ai: AiSettings,
    pub ui: UiSettings,
}

fn load_section<T: serde::de::DeserializeOwned + Default>(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    key: &str,
) -> T {
    store
        .get(key)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Migrate settings from v1 to v2 schema.
///
/// v1 → v2: removes `ollamaUrl`, `ollamaModel`, `openaiModel` from `ai`;
/// adds `endpoints: Vec<LlmEndpoint>`. Migrates `defaultLlmProvider` if it
/// was `"openai"` or `"ollama"`. Best-effort migrates the OpenAI keychain entry.
fn migrate_v1_to_v2(store: &tauri_plugin_store::Store<tauri::Wry>) {
    let old_ai = match store.get("ai") {
        Some(v) => v,
        None => {
            // No ai section; just bump the version.
            store.set("_schemaVersion", serde_json::json!(SCHEMA_VERSION));
            let _ = store.save();
            return;
        }
    };

    let ollama_url = old_ai
        .get("ollamaUrl")
        .and_then(|v| v.as_str())
        .unwrap_or("http://localhost:11434")
        .to_string();
    let ollama_model = old_ai
        .get("ollamaModel")
        .and_then(|v| v.as_str())
        .unwrap_or("llava:7b")
        .to_string();
    let openai_model = old_ai
        .get("openaiModel")
        .and_then(|v| v.as_str())
        .unwrap_or("gpt-4o")
        .to_string();
    let default_provider = old_ai
        .get("defaultLlmProvider")
        .and_then(|v| v.as_str())
        .unwrap_or("claude")
        .to_string();

    // Use stable well-known IDs for the migrated endpoints.
    let openai_id = "openai".to_string();
    let ollama_id = "ollama-local".to_string();

    // Normalise the Ollama URL: ensure it ends with /v1.
    let ollama_base_url = if ollama_url.trim_end_matches('/').ends_with("/v1") {
        ollama_url.trim_end_matches('/').to_string()
    } else {
        format!("{}/v1", ollama_url.trim_end_matches('/'))
    };

    let openai_endpoint = LlmEndpoint {
        id: openai_id.clone(),
        name: "OpenAI".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        model: openai_model,
    };
    let ollama_endpoint = LlmEndpoint {
        id: ollama_id.clone(),
        name: "Ollama (local)".to_string(),
        base_url: ollama_base_url,
        model: ollama_model,
    };

    let new_default_provider = match default_provider.as_str() {
        "openai" => format!("endpoint:{openai_id}"),
        "ollama" => format!("endpoint:{ollama_id}"),
        other => other.to_string(),
    };

    // Preserve all other ai fields across the migration.
    let claude_model = old_ai
        .get("claudeModel")
        .and_then(|v| v.as_str())
        .unwrap_or("claude-sonnet-4-20250514")
        .to_string();
    let gemini_model = old_ai
        .get("geminiModel")
        .and_then(|v| v.as_str())
        .unwrap_or("gemini-2.0-flash")
        .to_string();
    let ocr_language = old_ai
        .get("ocrLanguage")
        .and_then(|v| v.as_str())
        .unwrap_or("eng")
        .to_string();
    let image_max_dim = old_ai
        .get("imageMaxDim")
        .and_then(|v| v.as_u64())
        .unwrap_or(2048) as u32;
    let image_quality = old_ai
        .get("imageQuality")
        .and_then(|v| v.as_u64())
        .unwrap_or(85) as u8;

    let new_ai = AiSettings {
        ocr_language,
        default_llm_provider: new_default_provider,
        endpoints: vec![openai_endpoint, ollama_endpoint],
        claude_model,
        gemini_model,
        image_max_dim,
        image_quality,
    };

    if let Ok(v) = serde_json::to_value(&new_ai) {
        store.set("ai", v);
    }

    // Best-effort: migrate OpenAI keychain entry to the new endpoint account.
    if let Ok(key) = crate::credentials::get_api_key("openai") {
        if !key.is_empty() {
            let _ = crate::credentials::store_api_key(&format!("endpoint:{openai_id}"), &key);
            let _ = crate::credentials::delete_api_key("openai");
        }
    }

    store.set("_schemaVersion", serde_json::json!(SCHEMA_VERSION));
    let _ = store.save();
}

/// Run any pending schema migrations before loading settings.
fn migrate_if_needed(store: &tauri_plugin_store::Store<tauri::Wry>) {
    let version = store
        .get("_schemaVersion")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if version < 2 {
        migrate_v1_to_v2(store);
    }
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let store = app
        .store(STORE_PATH)
        .map_err(|e| format!("Store error: {e}"))?;
    migrate_if_needed(&store);
    Ok(Settings {
        capture: load_section(&store, "capture"),
        annotation: load_section(&store, "annotation"),
        ai: load_section(&store, "ai"),
        ui: load_section(&store, "ui"),
    })
}

#[tauri::command]
pub fn set_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let store = app
        .store(STORE_PATH)
        .map_err(|e| format!("Store error: {e}"))?;
    store.set(
        "capture",
        serde_json::to_value(&settings.capture).map_err(|e| e.to_string())?,
    );
    store.set(
        "annotation",
        serde_json::to_value(&settings.annotation).map_err(|e| e.to_string())?,
    );
    store.set(
        "ai",
        serde_json::to_value(&settings.ai).map_err(|e| e.to_string())?,
    );
    store.set(
        "ui",
        serde_json::to_value(&settings.ui).map_err(|e| e.to_string())?,
    );
    store.set("_schemaVersion", serde_json::json!(SCHEMA_VERSION));
    store.save().map_err(|e| format!("Save error: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn set_api_key(provider: String, key: String) -> Result<(), String> {
    crate::credentials::store_api_key(&provider, &key)
        .map_err(|e| format!("Failed to store API key: {e}"))
}

#[tauri::command]
pub fn get_api_key(provider: String) -> Result<String, String> {
    match crate::credentials::get_api_key(&provider) {
        Ok(key) => {
            // Return masked form: 8 bullets + last 4 chars (or all bullets if short)
            let masked = if key.len() > 4 {
                format!(
                    "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}{}",
                    &key[key.len() - 4..]
                )
            } else {
                "\u{2022}".repeat(key.len())
            };
            Ok(masked)
        }
        // No entry = not an error, just no key set
        Err(_) => Ok(String::new()),
    }
}

#[tauri::command]
pub fn delete_api_key(provider: String) -> Result<(), String> {
    match crate::credentials::delete_api_key(&provider) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Treat missing entry as success
            if matches!(
                e.downcast_ref::<keyring::Error>(),
                Some(keyring::Error::NoEntry)
            ) {
                Ok(())
            } else {
                Err(format!("Failed to delete API key: {e}"))
            }
        }
    }
}

#[tauri::command]
pub async fn test_api_key(app: tauri::AppHandle, provider: String) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    match provider.as_str() {
        "anthropic" => {
            let key = crate::credentials::get_api_key(&provider)
                .map_err(|_| "No API key configured for 'anthropic'".to_string())?;
            if key.is_empty() {
                return Err("No API key configured for 'anthropic'".to_string());
            }
            let status = client
                .get("https://api.anthropic.com/v1/models")
                .header("x-api-key", &key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
                .map_err(|e| format!("Request failed: {e}"))?
                .status();
            check_key_status(status)
        }
        "gemini" => {
            let key = crate::credentials::get_api_key(&provider)
                .map_err(|_| "No API key configured for 'gemini'".to_string())?;
            if key.is_empty() {
                return Err("No API key configured for 'gemini'".to_string());
            }
            let url =
                format!("https://generativelanguage.googleapis.com/v1/models?key={key}");
            let status = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {e}"))?
                .status();
            check_key_status(status)
        }
        s if s.starts_with("endpoint:") => {
            let id = s["endpoint:".len()..].to_string();
            let store = app
                .store(STORE_PATH)
                .map_err(|e| format!("Store error: {e}"))?;
            let ai_settings: AiSettings = store
                .get("ai")
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let endpoint = ai_settings
                .endpoints
                .iter()
                .find(|e| e.id == id)
                .ok_or_else(|| format!("Unknown endpoint '{id}'"))?;

            let api_key = crate::credentials::get_api_key(&provider).unwrap_or_default();
            let base = endpoint.base_url.trim_end_matches('/');
            let url = format!("{base}/models");

            let mut req = client.get(&url);
            if !api_key.is_empty() {
                req = req.header(header::AUTHORIZATION, format!("Bearer {api_key}"));
            }
            let status = req
                .send()
                .await
                .map_err(|e| format!("Request failed: {e}"))?
                .status();
            if status.is_success() {
                Ok(())
            } else {
                Err(format!("API returned status {status}"))
            }
        }
        other => Err(format!("Unknown provider '{other}'")),
    }
}

fn check_key_status(status: reqwest::StatusCode) -> Result<(), String> {
    if status.is_success() {
        Ok(())
    } else if status.as_u16() == 401 || status.as_u16() == 403 {
        Err(format!("Authentication failed ({status}): invalid API key"))
    } else {
        Err(format!("API returned unexpected status {status}"))
    }
}
