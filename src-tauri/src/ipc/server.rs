/// IPC server — runs inside the main Tauri app process.
///
/// Binds a Unix socket at `$XDG_RUNTIME_DIR/fotos-ipc.sock` (fallback:
/// `/tmp/fotos-ipc.sock`) and accepts connections from `fotos-mcp`.
///
/// Protocol: each message is framed as a 4-byte big-endian u32 payload length
/// followed by that many bytes of UTF-8 JSON.  Request: `{id, command, params}`.
/// Response: `{id, ok}` on success or `{id, error: {code, message}}` on failure.
use anyhow::Result;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tracing::{error, info, warn};
use uuid::Uuid;

#[cfg(unix)]
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

// ─── wire types ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct IpcRequest {
    id: String,
    command: String,
    params: Value,
}

#[derive(Serialize)]
struct IpcResponse {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ok: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<IpcError>,
}

#[derive(Serialize)]
struct IpcError {
    code: String,
    message: String,
}

impl IpcResponse {
    fn ok(id: String, value: Value) -> Self {
        Self {
            id,
            ok: Some(value),
            error: None,
        }
    }
    fn err(id: String, code: &str, message: String) -> Self {
        Self {
            id,
            ok: None,
            error: Some(IpcError {
                code: code.to_owned(),
                message,
            }),
        }
    }
}

// ─── socket path ─────────────────────────────────────────────────────────────

pub fn socket_path() -> PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
    PathBuf::from(base).join("fotos-ipc.sock")
}

// ─── server entry point ──────────────────────────────────────────────────────

/// Start the IPC server. This is a long-running async task; spawn it with
/// `tauri::async_runtime::spawn`.
pub async fn start_ipc_server(app: tauri::AppHandle) -> Result<()> {
    #[cfg(not(unix))]
    {
        warn!("IPC server not yet supported on this platform — skipping");
        return Ok(());
    }

    #[cfg(unix)]
    {
        let path = socket_path();
        // Remove a stale socket from a previous run.
        let _ = std::fs::remove_file(&path);

        let listener = UnixListener::bind(&path)?;
        info!("IPC server listening at {}", path.display());

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let app = app.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, app).await {
                            warn!("IPC connection closed: {e}");
                        }
                    });
                }
                Err(e) => {
                    error!("IPC accept error: {e}");
                }
            }
        }
    }
}

// ─── per-connection handler ───────────────────────────────────────────────────

#[cfg(unix)]
async fn handle_connection(mut stream: UnixStream, app: tauri::AppHandle) -> Result<()> {
    loop {
        // Read the 4-byte length prefix.
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf).await {
            Ok(_) => {}
            // Clean EOF — client disconnected.
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let body_len = u32::from_be_bytes(len_buf) as usize;
        let mut body = vec![0u8; body_len];
        stream.read_exact(&mut body).await?;

        let response = match serde_json::from_slice::<IpcRequest>(&body) {
            Ok(req) => {
                let id = req.id.clone();
                match dispatch(&app, &req.command, req.params).await {
                    Ok(v) => IpcResponse::ok(id, v),
                    Err(e) => IpcResponse::err(id, "command_error", e.to_string()),
                }
            }
            Err(e) => {
                // Malformed request — id unknown, use empty string.
                warn!("IPC: malformed request: {e}");
                IpcResponse::err(String::new(), "invalid_request", e.to_string())
            }
        };

        let payload = serde_json::to_vec(&response)?;
        let len = u32::try_from(payload.len())?.to_be_bytes();
        stream.write_all(&len).await?;
        stream.write_all(&payload).await?;
    }
    Ok(())
}

// ─── command dispatcher ───────────────────────────────────────────────────────

async fn dispatch(app: &tauri::AppHandle, command: &str, params: Value) -> anyhow::Result<Value> {
    match command {
        "get_settings" => {
            let settings = crate::commands::settings::get_settings(app.clone())
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(serde_json::to_value(settings)?)
        }

        "take_screenshot" => {
            let mode = params
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("fullscreen")
                .to_owned();
            let image = match mode.as_str() {
                "fullscreen" => {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.hide();
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    let img = crate::capture::xcap_backend::capture_fullscreen()
                        .await
                        .map_err(|e| anyhow::anyhow!("Capture failed: {e}"))?;
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                    }
                    img
                }
                "monitor" => {
                    let idx = params
                        .get("monitor_index")
                        .and_then(Value::as_u64)
                        .map(|v| v as u32)
                        .ok_or_else(|| {
                            anyhow::anyhow!("monitor_index required for mode 'monitor'")
                        })?;
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.hide();
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    let img = crate::capture::xcap_backend::capture_monitor(idx)
                        .await
                        .map_err(|e| anyhow::anyhow!("Monitor capture failed: {e}"))?;
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                    }
                    img
                }
                "window" => {
                    let title_sub = params
                        .get("window_title")
                        .and_then(Value::as_str)
                        .ok_or_else(|| anyhow::anyhow!("window_title required for mode 'window'"))?
                        .to_lowercase();
                    let windows = crate::commands::capture::list_windows()
                        .await
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    let win = windows
                        .into_iter()
                        .find(|w| w.title.to_lowercase().contains(&title_sub))
                        .ok_or_else(|| anyhow::anyhow!("No window matching title"))?;
                    crate::capture::xcap_backend::capture_window(win.id)
                        .await
                        .map_err(|e| anyhow::anyhow!("Window capture failed: {e}"))?
                }
                other => anyhow::bail!("Unknown capture mode '{other}'"),
            };
            let image = Arc::new(image);
            let (width, height) = (image.width(), image.height());
            let id = Uuid::new_v4();
            app.state::<crate::capture::ImageStore>()
                .insert(id, Arc::clone(&image));
            let mut png_data = Vec::new();
            image
                .write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
                .map_err(|e| anyhow::anyhow!("PNG encoding failed: {e}"))?;
            Ok(serde_json::json!({
                "id": id.to_string(),
                "image_b64": BASE64_STANDARD.encode(&png_data),
                "width": width,
                "height": height,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "mode": mode,
            }))
        }

        "ocr_screenshot" => {
            let lang = params
                .get("language")
                .and_then(Value::as_str)
                .unwrap_or("eng")
                .to_owned();
            let store = app.state::<crate::capture::ImageStore>();
            let (image, screenshot_id) = match params.get("screenshot_id").and_then(Value::as_str) {
                Some(id_str) => {
                    let uuid = Uuid::parse_str(id_str).map_err(|e| anyhow::anyhow!("{e}"))?;
                    let img = store
                        .get(&uuid)
                        .ok_or_else(|| anyhow::anyhow!("Screenshot not found: {id_str}"))?;
                    (img, uuid)
                }
                None => {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.hide();
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    let img = crate::capture::xcap_backend::capture_fullscreen()
                        .await
                        .map_err(|e| anyhow::anyhow!("Capture failed: {e}"))?;
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                    }
                    let img = Arc::new(img);
                    let id = Uuid::new_v4();
                    store.insert(id, Arc::clone(&img));
                    (img, id)
                }
            };
            let tessdata_path = crate::commands::ai::resolve_tessdata_path(app, &lang)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let opts = crate::ai::ocr::OcrOptions {
                lang,
                tessdata_path,
            };
            let progress_app = app.clone();
            let on_progress = move |current: u32, total: u32| {
                let _ = tauri::Emitter::emit(
                    &progress_app,
                    "ocr:progress",
                    serde_json::json!({"current": current, "total": total}),
                );
            };
            let ocr = crate::ai::ocr::run_ocr(&image, &opts, Some(&on_progress))
                .map_err(|e| anyhow::anyhow!("OCR failed: {e}"))?;
            Ok(serde_json::json!({
                "screenshot_id": screenshot_id.to_string(),
                "text": ocr.full_text,
                "regions": ocr.regions.into_iter().map(|r| serde_json::json!({
                    "text": r.text, "x": r.x, "y": r.y, "w": r.w, "h": r.h,
                    "confidence": r.confidence,
                })).collect::<Vec<_>>(),
            }))
        }

        "annotate_screenshot" => {
            let id_str = params
                .get("screenshot_id")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("screenshot_id required"))?
                .to_owned();
            let annotations_val = inject_annotation_ids(
                params
                    .get("annotations")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
            );
            let annotations: Vec<crate::commands::files::Annotation> =
                serde_json::from_value(annotations_val)
                    .map_err(|e| anyhow::anyhow!("Invalid annotations: {e}"))?;
            let store = app.state::<crate::capture::ImageStore>();
            let image_b64 =
                crate::commands::files::composite_image(id_str, annotations, None, store)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(serde_json::json!({ "image_b64": image_b64 }))
        }

        "analyze_screenshot" => {
            use crate::ai::{compress, llm};
            use tauri_plugin_store::StoreExt;

            let prompt = params
                .get("prompt")
                .and_then(Value::as_str)
                .map(str::to_owned);
            let provider = params
                .get("provider")
                .and_then(Value::as_str)
                .unwrap_or("claude")
                .to_owned();

            let store = app.state::<crate::capture::ImageStore>();
            let (image, _id) = match params.get("screenshot_id").and_then(Value::as_str) {
                Some(id_str) => {
                    let uuid = Uuid::parse_str(id_str).map_err(|e| anyhow::anyhow!("{e}"))?;
                    let img = store
                        .get(&uuid)
                        .ok_or_else(|| anyhow::anyhow!("Screenshot not found: {id_str}"))?;
                    (img, uuid)
                }
                None => {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.hide();
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    let img = crate::capture::xcap_backend::capture_fullscreen()
                        .await
                        .map_err(|e| anyhow::anyhow!("Capture failed: {e}"))?;
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                    }
                    let img = Arc::new(img);
                    let id = Uuid::new_v4();
                    store.insert(id, Arc::clone(&img));
                    (img, id)
                }
            };

            let prefs_store = app
                .store("prefs.json")
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let ai: crate::commands::settings::AiSettings = prefs_store
                .get("ai")
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            let image_b64 = compress::compress_for_llm(&image, ai.image_max_dim, ai.image_quality)
                .map_err(|e| anyhow::anyhow!("Compression failed: {e}"))?;
            let prompt_text = prompt.unwrap_or_else(|| "Describe this image.".to_owned());

            let output = match provider.as_str() {
                "claude" | "anthropic" => {
                    let key = crate::credentials::get_api_key("anthropic")
                        .map_err(|_| anyhow::anyhow!("No Anthropic API key configured"))?;
                    llm::analyze(
                        &image_b64,
                        &prompt_text,
                        &llm::LlmProvider::Claude {
                            model: ai.claude_model,
                        },
                        &key,
                    )
                    .await?
                }
                "gemini" => {
                    let key = crate::credentials::get_api_key("gemini")
                        .map_err(|_| anyhow::anyhow!("No Gemini API key configured"))?;
                    llm::analyze(
                        &image_b64,
                        &prompt_text,
                        &llm::LlmProvider::Gemini {
                            model: ai.gemini_model,
                        },
                        &key,
                    )
                    .await?
                }
                s if s.starts_with("endpoint:") => {
                    let id = &s["endpoint:".len()..];
                    let endpoint = ai
                        .endpoints
                        .iter()
                        .find(|e| e.id == id)
                        .ok_or_else(|| anyhow::anyhow!("Unknown endpoint '{id}'"))?;
                    let api_key =
                        crate::credentials::get_api_key(provider.as_str()).unwrap_or_default();
                    crate::ai::openai_compat::analyze(
                        &image_b64,
                        &prompt_text,
                        &endpoint.base_url,
                        &endpoint.model,
                        &api_key,
                    )
                    .await?
                }
                other => anyhow::bail!("Unknown provider '{other}'"),
            };
            Ok(serde_json::json!({
                "provider": provider,
                "model": output.model,
                "response_text": output.response,
                "tokens_used": output.tokens_used,
                "latency_ms": output.latency_ms,
            }))
        }

        "auto_redact_pii" => {
            let id_str = params
                .get("screenshot_id")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("screenshot_id required"))?
                .to_owned();
            let store = app.state::<crate::capture::ImageStore>();
            let uuid = Uuid::parse_str(&id_str).map_err(|e| anyhow::anyhow!("{e}"))?;
            let image = store
                .get(&uuid)
                .ok_or_else(|| anyhow::anyhow!("Screenshot not found: {id_str}"))?;

            let tessdata_path = crate::commands::ai::resolve_tessdata_path(app, "eng")
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let opts = crate::ai::ocr::OcrOptions {
                lang: "eng".to_owned(),
                tessdata_path,
            };
            let ocr = crate::ai::ocr::run_ocr(&image, &opts, None)
                .map_err(|e| anyhow::anyhow!("OCR failed: {e}"))?;
            let pii = crate::ai::pii::detect_pii(&ocr.regions)
                .map_err(|e| anyhow::anyhow!("PII detection failed: {e}"))?;

            let blur_annotations: Vec<crate::commands::files::Annotation> = pii
                .iter()
                .map(|m| crate::commands::files::Annotation {
                    id: Uuid::new_v4().to_string(),
                    annotation_type: "blur".to_owned(),
                    x: m.x as f64,
                    y: m.y as f64,
                    width: Some(m.w as f64),
                    height: Some(m.h as f64),
                    stroke_color: None,
                    fill_color: None,
                    stroke_width: None,
                    opacity: None,
                    text: None,
                    font_size: None,
                    font_family: None,
                    points: None,
                    step_number: None,
                    blur_radius: None,
                    highlight_color: None,
                    created_at: None,
                    locked: None,
                })
                .collect();

            let image_b64 =
                crate::commands::files::composite_image(id_str, blur_annotations, None, store)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;

            let detections: Vec<Value> = pii
                .into_iter()
                .map(|m| {
                    serde_json::json!({
                        "type": m.pii_type, "x": m.x, "y": m.y, "w": m.w, "h": m.h,
                    })
                })
                .collect();
            Ok(serde_json::json!({ "image_b64": image_b64, "detections": detections }))
        }

        "list_screenshots" => {
            let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
            let ids = app.state::<crate::capture::ImageStore>().ids();
            let entries: Vec<Value> = ids
                .into_iter()
                .take(limit)
                .map(|id| serde_json::json!({ "id": id.to_string() }))
                .collect();
            Ok(Value::Array(entries))
        }

        _ => Err(anyhow::anyhow!("unknown command: {command}")),
    }
}

/// Inject a random `id` field into any annotation object that is missing one.
fn inject_annotation_ids(mut val: Value) -> Value {
    if let Some(arr) = val.as_array_mut() {
        for item in arr.iter_mut() {
            if let Some(obj) = item.as_object_mut() {
                if !obj.contains_key("id") {
                    obj.insert("id".to_owned(), Value::String(Uuid::new_v4().to_string()));
                }
            }
        }
    }
    val
}
