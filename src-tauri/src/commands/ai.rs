use crate::ai::ocr::OcrOptions;
use crate::capture::ImageStore;
use serde::Serialize;
use std::path::PathBuf;
use tauri::Emitter;
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

#[derive(Clone, Serialize)]
pub struct OcrProgressPayload {
    pub current: u32,
    pub total: u32,
}

#[derive(Serialize)]
pub struct LlmResponse {
    pub provider: String,
    pub model: String,
    pub response_text: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
}

/// Resolve the tessdata directory path for the given language.
/// - "eng" uses the bundled tessdata (or the Flatpak-provided path).
/// - Other languages use the app data directory where traineddata files
///   are downloaded on demand.
pub fn resolve_tessdata_path(app: &tauri::AppHandle, lang: &str) -> Result<String, String> {
    use tauri::Manager;

    // Non-English langs always live in the app data directory.
    if lang != "eng" {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {e}"))?
            .join("tessdata");
        return dir
            .to_str()
            .ok_or_else(|| "tessdata path contains invalid UTF-8".to_string())
            .map(|s| s.to_string());
    }

    // English: use the bundled tessdata.
    if std::env::var("FLATPAK_ID").is_ok() {
        return Ok("/app/share/tessdata".to_string());
    }
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

/// Return whether the traineddata file for `lang` is available locally.
#[tauri::command]
pub fn tessdata_available(app: tauri::AppHandle, lang: String) -> Result<bool, String> {
    use tauri::Manager;
    if lang == "eng" {
        // English is always bundled.
        return Ok(true);
    }
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?
        .join("tessdata")
        .join(format!("{lang}.traineddata"));
    Ok(path.exists())
}

#[derive(Clone, serde::Serialize)]
pub struct TessdataProgressPayload {
    pub lang: String,
    pub downloaded: u64,
    pub total: u64,
}

/// Download the tessdata file for `lang` from the Tesseract GitHub release.
/// No-ops if the file is already present. Emits `tessdata:progress` events
/// with `{ lang, downloaded, total }` (total=0 when content-length is unknown).
#[tauri::command]
pub async fn download_tessdata(app: tauri::AppHandle, lang: String) -> Result<(), String> {
    use tauri::{Emitter, Manager};

    match lang.as_str() {
        "fra" | "deu" | "spa" => {}
        other => return Err(format!("Unsupported tessdata language: {other}")),
    }

    let tessdata_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?
        .join("tessdata");

    std::fs::create_dir_all(&tessdata_dir)
        .map_err(|e| format!("Failed to create tessdata dir: {e}"))?;

    let dest = tessdata_dir.join(format!("{lang}.traineddata"));
    if dest.exists() {
        return Ok(());
    }

    let url =
        format!("https://raw.githubusercontent.com/tesseract-ocr/tessdata/main/{lang}.traineddata");

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let total = response.content_length().unwrap_or(0);

    // Emit an initial progress event so the UI can show "Downloading…".
    let _ = app.emit(
        "tessdata:progress",
        TessdataProgressPayload {
            lang: lang.clone(),
            downloaded: 0,
            total,
        },
    );

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download body: {e}"))?;

    let downloaded = bytes.len() as u64;
    std::fs::write(&dest, &bytes).map_err(|e| format!("Failed to write tessdata: {e}"))?;

    let _ = app.emit(
        "tessdata:progress",
        TessdataProgressPayload {
            lang: lang.clone(),
            downloaded,
            total: downloaded,
        },
    );

    Ok(())
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

    let lang = lang.unwrap_or_else(|| "eng".to_string());
    let tessdata_path = resolve_tessdata_path(&app, &lang)?;
    let opts = OcrOptions {
        lang,
        tessdata_path,
    };

    let progress_app = app.clone();
    let on_progress = move |current: u32, total: u32| {
        let _ = progress_app.emit("ocr:progress", OcrProgressPayload { current, total });
    };
    let output = crate::ai::ocr::run_ocr(&image, &opts, Some(&on_progress))
        .map_err(|e| format!("OCR failed: {e}"))?;

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

    let tessdata_path = resolve_tessdata_path(&app, "eng")?;
    let opts = OcrOptions {
        lang: "eng".to_string(),
        tessdata_path,
    };

    let ocr_output =
        crate::ai::ocr::run_ocr(&image, &opts, None).map_err(|e| format!("OCR failed: {e}"))?;

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
    use crate::ai::{compress, llm, openai_compat};
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
        s if s.starts_with("endpoint:") => {
            let id = &s["endpoint:".len()..];
            let endpoint = ai_settings
                .endpoints
                .iter()
                .find(|e| e.id == id)
                .ok_or_else(|| format!("Unknown endpoint '{id}'"))?;
            let api_key = crate::credentials::get_api_key(&provider).unwrap_or_default();
            openai_compat::analyze(
                &image_b64,
                &prompt_text,
                &endpoint.base_url,
                &endpoint.model,
                &api_key,
            )
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
