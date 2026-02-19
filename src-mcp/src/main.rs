use tracing_subscriber;

mod bridge;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // TODO: start MCP server on stdio transport
    tracing::info!("fotos-mcp starting");

    Ok(())
}
