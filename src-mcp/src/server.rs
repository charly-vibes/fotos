/// MCP server implementation.
///
/// Defines tools, resources, and prompts exposed to MCP hosts.
/// Delegates actual work to the main Fotos app via IPC bridge.
#[allow(dead_code)]
pub struct FotosMcpServer {
    // TODO: hold IPC connection to main app
}

#[allow(dead_code)]
impl FotosMcpServer {
    pub fn new() -> Self {
        Self {}
    }

    // Tools:
    // - take_screenshot
    // - ocr_screenshot
    // - annotate_screenshot
    // - analyze_screenshot
    // - auto_redact_pii
    // - list_screenshots

    // Resources:
    // - screenshots://recent
    // - screenshots://{id}
    // - screenshots://{id}/ocr
    // - settings://current

    // Prompts:
    // - describe_ui
    // - extract_code
    // - generate_bug_report
    // - accessibility_audit
}
