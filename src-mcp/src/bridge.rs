/// IPC bridge — connects the MCP server to the running Tauri app.
///
/// Protocol: 4-byte big-endian u32 length prefix + UTF-8 JSON body.
/// Request:  `{id, command, params}` — Response: `{id, ok?} | {id, error?}`
/// Socket:   `$XDG_RUNTIME_DIR/fotos-ipc.sock`  (fallback: `/tmp/fotos-ipc.sock`)
///
/// Call `AppBridge::connect()` and handle the error to gracefully fall back to
/// standalone mode when the main Fotos app is not running.
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

#[cfg(unix)]
use tokio::net::UnixStream;

#[allow(dead_code)]
#[derive(Serialize)]
struct IpcRequest {
    id: String,
    command: String,
    params: Value,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct IpcResponse {
    id: String,
    ok: Option<Value>,
    error: Option<IpcError>,
}

#[derive(Deserialize)]
struct IpcError {
    code: String,
    message: String,
}

/// Client half of the MCP ↔ Tauri IPC channel.
#[allow(dead_code)]
///
/// All requests are sent over a single persistent connection. The stream is
/// protected by a `Mutex` so concurrent callers queue naturally (the MCP spec
/// mandates sequential tool execution anyway).
#[derive(Clone)]
pub struct AppBridge {
    #[cfg(unix)]
    stream: Arc<Mutex<UnixStream>>,
}

#[allow(dead_code)]
impl AppBridge {
    /// Connect to the running Tauri app with a 2-second timeout.
    /// Returns `Err` if the app is not running — callers should fall back to
    /// standalone mode.
    pub async fn connect() -> Result<Self> {
        #[cfg(unix)]
        {
            let path = socket_path();
            let stream =
                timeout(Duration::from_secs(2), UnixStream::connect(&path))
                    .await
                    .map_err(|_| {
                        anyhow!(
                            "timeout connecting to Fotos app at {} — is it running?",
                            path.display()
                        )
                    })?
                    .map_err(|e| {
                        anyhow!("failed to connect to {}: {e}", path.display())
                    })?;

            tracing::info!("IPC bridge connected to {}", path.display());
            return Ok(Self { stream: Arc::new(Mutex::new(stream)) });
        }

        #[cfg(not(unix))]
        Err(anyhow!("IPC bridge not yet supported on this platform"))
    }

    /// Send a command to the Tauri app and wait for the response (30 s timeout).
    pub async fn send_command(&self, command: &str, params: Value) -> Result<Value> {
        #[cfg(unix)]
        {
            // Generate a simple request ID.
            let id = format!("{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0));

            let req = IpcRequest { id: id.clone(), command: command.to_owned(), params };
            let payload = serde_json::to_vec(&req)?;

            let mut stream = self.stream.lock().await;

            // Write: 4-byte big-endian length + payload
            let len = u32::try_from(payload.len())
                .map_err(|_| anyhow!("IPC payload too large ({} bytes)", payload.len()))?;
            stream.write_all(&len.to_be_bytes()).await?;
            stream.write_all(&payload).await?;

            // Read response length
            let mut len_buf = [0u8; 4];
            timeout(Duration::from_secs(30), stream.read_exact(&mut len_buf))
                .await
                .map_err(|_| anyhow!("timeout waiting for IPC response to '{command}'"))??;

            // Read response body
            let resp_len = u32::from_be_bytes(len_buf) as usize;
            let mut resp_buf = vec![0u8; resp_len];
            timeout(Duration::from_secs(30), stream.read_exact(&mut resp_buf))
                .await
                .map_err(|_| anyhow!("timeout reading IPC response body"))??;

            let resp: IpcResponse = serde_json::from_slice(&resp_buf)?;
            if resp.id != id {
                return Err(anyhow!(
                    "IPC response id mismatch: expected {id}, got {}",
                    resp.id
                ));
            }

            return match (resp.ok, resp.error) {
                (Some(ok), _) => Ok(ok),
                (_, Some(err)) => Err(anyhow!("[{}] {}", err.code, err.message)),
                _ => Err(anyhow!("invalid IPC response: missing ok/error fields")),
            };
        }

        #[allow(unreachable_code)]
        Err(anyhow!("IPC bridge not yet supported on this platform"))
    }
}

/// Returns the Unix socket path, respecting `$XDG_RUNTIME_DIR`.
#[allow(dead_code)]
pub fn socket_path() -> PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
    PathBuf::from(base).join("fotos-ipc.sock")
}
