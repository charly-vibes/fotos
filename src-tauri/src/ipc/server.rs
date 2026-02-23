/// IPC server — runs in the main Tauri app process.
///
/// Listens on a Unix socket (Linux) or named pipe (Windows) for
/// commands from the MCP server process.
use anyhow::Result;

pub async fn start_ipc_server() -> Result<()> {
    // TODO: create local socket/pipe and listen for MCP commands
    anyhow::bail!("IPC server not yet implemented")
}
