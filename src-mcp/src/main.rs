mod bridge;
mod prompts;
mod server;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{fmt, EnvFilter};

use server::FotosMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Log to stderr — stdout is reserved for the JSON-RPC transport
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("fotos-mcp v{} starting", env!("CARGO_PKG_VERSION"));

    let service = FotosMcpServer::new();
    let running = service.serve(stdio()).await?;

    tracing::info!("MCP session started, waiting for client");
    let quit = running.waiting().await?;
    tracing::info!("MCP session ended: {:?}", quit);

    Ok(())
}
