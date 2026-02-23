/// IPC client — used by the MCP server to communicate with the main app.
///
/// Connects to the Unix socket (Linux) or named pipe (Windows) that
/// the main app's IPC server listens on.
use anyhow::Result;

pub async fn connect_to_app() -> Result<()> {
    // TODO: connect to the main app's IPC socket/pipe
    anyhow::bail!("IPC client not yet implemented")
}
