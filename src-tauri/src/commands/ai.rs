use crate::capture::ImageStore;
use serde::Serialize;
use std::path::PathBuf;
use tesseract::Tesseract;
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

#[tauri::command]
pub fn run_ocr(
    app: tauri::AppHandle,
    image_id: String,
    lang: Option<String>,
    store: tauri::State<'_, ImageStore>,
) -> Result<OcrResult, String> {
    // Parse UUID
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {}", e))?;

    // Get image from store
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    // Convert to RGB8 format (tesseract expects raw pixel data)
    let rgb_image = image.to_rgb8();
    let width = rgb_image.width();
    let height = rgb_image.height();
    let raw_data = rgb_image.into_raw();

    let lang_str = lang.unwrap_or_else(|| "eng".to_string());

    // Convert u32 dimensions to i32 for Tesseract API
    let width_i32 = width as i32;
    let height_i32 = height as i32;
    let bytes_per_line = (width * 3) as i32;

    // Determine tessdata path.
    // - Flatpak: tessdata is installed at /app/share/tessdata/
    // - Bundled app: tessdata is in the app resource directory
    let bundled_path: PathBuf;
    let tessdata_path: &str = if std::env::var("FLATPAK_ID").is_ok() {
        "/app/share/tessdata"
    } else {
        use tauri::Manager;
        bundled_path = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?
            .join("resources")
            .join("tessdata");
        bundled_path
            .to_str()
            .ok_or_else(|| "tessdata path contains invalid UTF-8".to_string())?
    };

    let mut tess = Tesseract::new(Some(tessdata_path), Some(&lang_str))
        .map_err(|e| {
            format!(
                "Failed to initialize Tesseract with language '{}': {}",
                lang_str, e
            )
        })?
        .set_frame(&raw_data, width_i32, height_i32, 3, bytes_per_line)
        .map_err(|e| format!("Failed to set image: {}", e))?
        .recognize()
        .map_err(|e| format!("OCR recognition failed: {}", e))?;

    let text = tess
        .get_text()
        .map_err(|e| format!("OCR get_text failed: {}", e))?;

    // Parse TSV output for per-word bounding boxes.
    // TSV columns: level page_num block_num par_num line_num word_num left top width height conf text
    // Level 5 = word. Skip entries with conf < 0 (non-word rows) or empty text.
    let tsv = tess
        .get_tsv_text(0)
        .map_err(|e| format!("OCR get_tsv_text failed: {}", e))?;

    let regions: Vec<OcrRegion> = tsv
        .lines()
        .skip(1) // skip header
        .filter_map(|line| {
            let cols: Vec<&str> = line.splitn(12, '\t').collect();
            if cols.len() < 12 {
                return None;
            }
            let level: u32 = cols[0].parse().ok()?;
            if level != 5 {
                return None;
            }
            let conf: f32 = cols[10].parse().ok()?;
            if conf < 0.0 {
                return None;
            }
            let word_text = cols[11].trim().to_string();
            if word_text.is_empty() {
                return None;
            }
            let x: u32 = cols[6].parse().ok()?;
            let y: u32 = cols[7].parse().ok()?;
            let w: u32 = cols[8].parse().ok()?;
            let h: u32 = cols[9].parse().ok()?;
            Some(OcrRegion {
                text: word_text,
                x,
                y,
                w,
                h,
                confidence: conf,
            })
        })
        .collect();

    Ok(OcrResult { text, regions })
}

#[tauri::command]
pub fn auto_blur_pii(
    app: tauri::AppHandle,
    image_id: String,
    store: tauri::State<'_, ImageStore>,
) -> Result<Vec<BlurRegion>, String> {
    use tauri::Manager;
    // Parse UUID
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {}", e))?;

    // Get image from store
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {}", image_id))?;

    // Convert to RGB8 for Tesseract
    let rgb_image = image.to_rgb8();
    let width = rgb_image.width();
    let height = rgb_image.height();
    let raw_data = rgb_image.into_raw();

    let width_i32 = width as i32;
    let height_i32 = height as i32;
    let bytes_per_line = (width * 3) as i32;

    // Resolve tessdata path
    let bundled_path: PathBuf;
    let tessdata_path: &str = if std::env::var("FLATPAK_ID").is_ok() {
        "/app/share/tessdata"
    } else {
        bundled_path = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?
            .join("resources")
            .join("tessdata");
        bundled_path
            .to_str()
            .ok_or_else(|| "tessdata path contains invalid UTF-8".to_string())?
    };

    // Run OCR with bounding boxes
    let mut tess = Tesseract::new(Some(tessdata_path), Some("eng"))
        .map_err(|e| format!("Failed to initialize Tesseract: {}", e))?
        .set_frame(&raw_data, width_i32, height_i32, 3, bytes_per_line)
        .map_err(|e| format!("Failed to set image: {}", e))?
        .recognize()
        .map_err(|e| format!("OCR recognition failed: {}", e))?;

    let tsv = tess
        .get_tsv_text(0)
        .map_err(|e| format!("OCR get_tsv_text failed: {}", e))?;

    // Parse TSV into ai::ocr::OcrRegion for PII detection
    let ocr_regions: Vec<crate::ai::ocr::OcrRegion> = tsv
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols: Vec<&str> = line.splitn(12, '\t').collect();
            if cols.len() < 12 {
                return None;
            }
            let level: u32 = cols[0].parse().ok()?;
            if level != 5 {
                return None;
            }
            let conf: f32 = cols[10].parse().ok()?;
            if conf < 0.0 {
                return None;
            }
            let word_text = cols[11].trim().to_string();
            if word_text.is_empty() {
                return None;
            }
            Some(crate::ai::ocr::OcrRegion {
                text: word_text,
                x: cols[6].parse().ok()?,
                y: cols[7].parse().ok()?,
                w: cols[8].parse().ok()?,
                h: cols[9].parse().ok()?,
                confidence: conf,
            })
        })
        .collect();

    // Run PII detection
    let pii_matches = crate::ai::pii::detect_pii(&ocr_regions)
        .map_err(|e| format!("PII detection failed: {}", e))?;

    // Convert PiiMatch → BlurRegion
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

    // Fetch image from store
    let uuid = Uuid::parse_str(&image_id).map_err(|e| format!("Invalid image ID: {e}"))?;
    let image = store
        .get(&uuid)
        .ok_or_else(|| format!("Image not found: {image_id}"))?;

    // Load AI settings for compression params and model selection
    let prefs_store = app
        .store("prefs.json")
        .map_err(|e| format!("Store error: {e}"))?;
    let ai_settings: crate::commands::settings::AiSettings = prefs_store
        .get("ai")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Compress image before sending to LLM
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
