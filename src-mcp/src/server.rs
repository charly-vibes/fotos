/// MCP server implementation.
///
/// Implements the ServerHandler trait from rmcp. The bridge is connected
/// lazily on first tool/resource call and cached for the session lifetime.
/// If the connection is lost, the next call re-attempts the connection.
use std::sync::Arc;

use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, GetPromptRequestParam, GetPromptResult,
        Implementation, InitializeRequestParam, ListPromptsResult, ListResourceTemplatesResult,
        ListResourcesResult, ListToolsResult, PaginatedRequestParam, ReadResourceRequestParam,
        ReadResourceResult, ServerCapabilities, ServerInfo,
    },
    service::{RequestContext, RoleServer},
    Error as McpError, ServerHandler,
};
use tokio::sync::Mutex;
use tracing::info;

use crate::bridge::AppBridge;
use crate::prompts;
use crate::resources;
use crate::tools;

#[derive(Clone)]
pub struct FotosMcpServer {
    /// Lazily-connected IPC bridge to the main Tauri app.
    /// `None` means not yet connected (or last connection failed).
    bridge: Arc<Mutex<Option<AppBridge>>>,
}

impl FotosMcpServer {
    pub fn new() -> Self {
        Self {
            bridge: Arc::new(Mutex::new(None)),
        }
    }

    /// Return a connected bridge, or `None` if the app is not running.
    ///
    /// Tries to connect if not already connected. On failure the guard is left
    /// as `None` so the next call will retry.
    async fn bridge(&self) -> Option<AppBridge> {
        let mut guard = self.bridge.lock().await;
        if guard.is_none() {
            match AppBridge::connect().await {
                Ok(b) => {
                    tracing::info!("IPC bridge connected");
                    *guard = Some(b);
                }
                Err(e) => {
                    tracing::debug!("IPC bridge unavailable: {e}");
                }
            }
        }
        guard.clone()
    }

    /// Clear the cached bridge so the next call will reconnect.
    async fn reset_bridge(&self) {
        *self.bridge.lock().await = None;
    }
}

impl ServerHandler for FotosMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "fotos-mcp".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
            instructions: Some(
                "Fotos MCP server: take and annotate screenshots, run OCR, redact PII.".to_owned(),
            ),
            ..Default::default()
        }
    }

    async fn initialize(
        &self,
        request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ServerInfo, McpError> {
        info!(
            client = %request.client_info.name,
            version = %request.client_info.version,
            protocol = ?request.protocol_version,
            "MCP client connected"
        );
        // Eagerly attempt bridge connection; failure is non-fatal.
        let _ = self.bridge().await;
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(tools::list())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let bridge = self.bridge().await;
        let result =
            tools::call(bridge.as_ref(), &request.name, request.arguments.as_ref()).await;
        // If we got an IPC error content block, reset so next call retries.
        if let Ok(ref r) = result {
            if r.is_error == Some(true)
                && r.content.iter().any(|c| format!("{c:?}").contains("IPC error"))
            {
                self.reset_bridge().await;
            }
        }
        result
    }

    async fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(prompts::list())
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        prompts::get(request)
    }

    async fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(resources::list())
    }

    async fn list_resource_templates(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(resources::list_templates())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let bridge = self.bridge().await;
        let result = resources::read(bridge.as_ref(), &request.uri).await;
        if result.is_err() {
            self.reset_bridge().await;
        }
        result
    }
}
