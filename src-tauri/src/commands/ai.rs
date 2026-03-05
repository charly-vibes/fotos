use crate::ai::ocr::OcrOptions;
use crate::capture::ImageStore;
use serde::Serialize;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize)]
pub struct OcrRegion {
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub confidence: f32,
}

#[derive(Serialize)]
pub struct OcrResult {
    pub text: String,
    pub regions: Vec<OcrRegion>,
}

#[derive(Serialize)]
pub struct BlurRegion {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pii_type: String,
}

#[derive(Serialize)]
pub struct LlmResponse {
    pub provider: String,
    pub model: String,
    pub response_text: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
}

/// Resolve the tessdata directory path for the current environment.
fn resolve_tessdata_path(app: &tauri::AppHandle) -> Result<String, String> {
    if std::env::var("FLATPAK_ID").is_ok() {
        return Ok("/app/share/tessdata".to_string());
    }
    use tauri::Manager;
    let path: PathBuf = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {e}"))?
        .join("resources")
        .join("tessdata");
    path.to_str()
        .ok_or_else(|| "tessdata path contains invalid UTF-8".to_string())
        .map(|s| s.to_string())
}

#[tauri::command]
pub fn run_ocr(
    app: tauri::AppHandle,
    image_id: String,
    lang: Option<String>,
    store: tauri::State<'_, ImageStore>,
) -> Result<OcrResult, String> {
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {e}"))?;
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {image_id}"))?;

    let tessdata_path = resolve_tessdata_path(&app)?;
    let opts = OcrOptions {
        lang: lang.unwrap_or_else(|| "eng".to_string()),
        tessdata_path,
    };

    let output =
        crate::ai::ocr::run_ocr(&image, &opts).map_err(|e| format!("OCR failed: {e}"))?;

    let regions = output
        .regions
        .into_iter()
        .map(|r| OcrRegion {
            text: r.text,
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
            confidence: r.confidence,
        })
        .collect();

    Ok(OcrResult {
        text: output.full_text,
        regions,
    })
}

#[tauri::command]
pub fn auto_blur_pii(
    app: tauri::AppHandle,
    image_id: String,
    store: tauri::State<'_, ImageStore>,
) -> Result<Vec<BlurRegion>, String> {
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {e}"))?;
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {image_id}"))?;

    let tessdata_path = resolve_tessdata_path(&app)?;
    let opts = OcrOptions {
        lang: "eng".to_string(),
        tessdata_path,
    };

    let ocr_output =
        crate::ai::ocr::run_ocr(&image, &opts).map_err(|e| format!("OCR failed: {e}"))?;

    let pii_matches = crate::ai::pii::detect_pii(&ocr_output.regions)
        .map_err(|e| format!("PII detection failed: {e}"))?;

    let blur_regions = pii_matches
        .into_iter()
        .map(|m| BlurRegion {
            x: m.x,
            y: m.y,
            w: m.w,
            h: m.h,
            pii_type: m.pii_type,
        })
        .collect();

    Ok(blur_regions)
}

#[tauri::command]
pub async fn analyze_llm(
    app: tauri::AppHandle,
    image_id: String,
    prompt: Option<String>,
    provider: String,
    store: tauri::State<'_, ImageStore>,
) -> Result<LlmResponse, String> {
    use crate::ai::{compress, llm, ollama};
    use tauri_plugin_store::StoreExt;

    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {e}"))?;
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {image_id}"))?;

    let prefs_store = app
        .store("prefs.json")
        .map_err(|e| format!("Store error: {e}"))?;
    let ai_settings: crate::commands::settings::AiSettings = prefs_store
        .get("ai")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let image_b64 =
        compress::compress_for_llm(&image, ai_settings.image_max_dim, ai_settings.image_quality)
            .map_err(|e| format!("Image compression failed: {e}"))?;

    let prompt_text = prompt.unwrap_or_else(|| "Describe this image.".to_string());

    let output = match provider.as_str() {
        "claude" | "anthropic" => {
            let api_key = crate::credentials::get_api_key("anthropic")
                .map_err(|_| "No Anthropic API key configured".to_string())?;
            let llm_provider = llm::LlmProvider::Claude {
                model: ai_settings.claude_model.clone(),
            };
            llm::analyze(&image_b64, &prompt_text, &llm_provider, &api_key)
                .await
                .map_err(|e| e.to_string())?
        }
        "openai" => {
            let api_key = crate::credentials::get_api_key("openai")
                .map_err(|_| "No OpenAI API key configured".to_string())?;
            let llm_provider = llm::LlmProvider::OpenAI {
                model: ai_settings.openai_model.clone(),
            };
            llm::analyze(&image_b64, &prompt_text, &llm_provider, &api_key)
                .await
                .map_err(|e| e.to_string())?
        }
        "gemini" => {
            let api_key = crate::credentials::get_api_key("gemini")
                .map_err(|_| "No Gemini API key configured".to_string())?;
            let llm_provider = llm::LlmProvider::Gemini {
                model: ai_settings.gemini_model.clone(),
            };
            llm::analyze(&image_b64, &prompt_text, &llm_provider, &api_key)
                .await
                .map_err(|e| e.to_string())?
        }
        "ollama" => {
            let config = ollama::OllamaConfig {
                url: ai_settings.ollama_url.clone(),
                model: ai_settings.ollama_model.clone(),
            };
            ollama::analyze(&image_b64, &prompt_text, &config)
                .await
                .map_err(|e| e.to_string())?
        }
        other => return Err(format!("Unknown provider '{other}'")),
    };

    Ok(LlmResponse {
        provider: provider.clone(),
        model: output.model,
        response_text: output.response,
        tokens_used: output.tokens_used,
        latency_ms: output.latency_ms,
    })
}
