/// MCP server implementation.
///
/// Implements the ServerHandler trait from rmcp. Prompt templates are fully
/// implemented here (fotos-kxs). Tools, resources, and IPC bridge are stubs
/// pending fotos-0j0, fotos-rsw, and fotos-d6e respectively.
use rmcp::{
    ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, GetPromptRequestParam, GetPromptResult,
        Implementation, InitializeRequestParam, ListPromptsResult, ListResourcesResult,
        ListToolsResult, PaginatedRequestParam, ReadResourceRequestParam, ReadResourceResult,
        ServerCapabilities, ServerInfo,
    },
    service::{RequestContext, RoleServer},
    Error as McpError,
};
use tracing::info;

use crate::bridge::AppBridge;
use crate::prompts;

#[derive(Clone)]
pub struct FotosMcpServer {
    #[allow(dead_code)]
    bridge: Option<AppBridge>,
}

impl FotosMcpServer {
    pub fn new() -> Self {
        Self { bridge: None }
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
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        // Populated in fotos-0j0
        Ok(ListToolsResult::default())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        Err(McpError::invalid_params(
            format!("unknown tool: {}", request.name),
            None,
        ))
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
        // Populated in fotos-rsw
        Ok(ListResourcesResult::default())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::invalid_params(
            format!("unknown resource: {}", request.uri),
            None,
        ))
    }
}
