/// IPC bridge to the main Fotos app.
///
/// Connects to the main app's IPC server (Unix socket on Linux,
/// named pipe on Windows) to delegate MCP tool calls.
use anyhow::Result;

#[allow(dead_code)]
pub struct AppBridge {
    // TODO: hold IPC connection
}

#[allow(dead_code)]
impl AppBridge {
    pub async fn connect() -> Result<Self> {
        // TODO: connect to main app IPC socket
        anyhow::bail!("IPC bridge not yet implemented")
    }

    pub async fn send_command(&self, _command: &str, _payload: &str) -> Result<String> {
        // TODO: send command via IPC and return response
        anyhow::bail!("IPC bridge not yet implemented")
    }
}
