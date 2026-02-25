/// IPC server — runs inside the main Tauri app process.
///
/// Binds a Unix socket at `$XDG_RUNTIME_DIR/fotos-ipc.sock` (fallback:
/// `/tmp/fotos-ipc.sock`) and accepts connections from `fotos-mcp`.
///
/// Protocol: each message is framed as a 4-byte big-endian u32 payload length
/// followed by that many bytes of UTF-8 JSON.  Request: `{id, command, params}`.
/// Response: `{id, ok}` on success or `{id, error: {code, message}}` on failure.
///
/// The dispatch table is extended in fotos-0j0 when the MCP tool implementations
/// are wired up.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tracing::{error, info, warn};

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
        Self { id, ok: Some(value), error: None }
    }
    fn err(id: String, code: &str, message: String) -> Self {
        Self {
            id,
            ok: None,
            error: Some(IpcError { code: code.to_owned(), message }),
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

/// Route an IPC command to its handler.
///
/// Currently handles only `get_settings`. Additional commands are added in
/// fotos-0j0 when the MCP tool implementations are wired up.
async fn dispatch(
    app: &tauri::AppHandle,
    command: &str,
    _params: Value,
) -> anyhow::Result<Value> {
    match command {
        "get_settings" => {
            let settings = crate::commands::settings::get_settings(app.clone())
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(serde_json::to_value(settings)?)
        }
        _ => Err(anyhow::anyhow!("unknown command: {command}")),
    }
}
