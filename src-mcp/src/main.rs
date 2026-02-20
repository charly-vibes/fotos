mod bridge;
mod server;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Print version to stderr and exit cleanly (tracer-bullet stub)
    eprintln!("fotos-mcp v{}", VERSION);
    eprintln!("MCP server stub - protocol implementation deferred");

    Ok(())
}
