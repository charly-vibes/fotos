/// MCP resource definitions and handlers.
///
/// Static resources: `screenshots://recent`, `settings://current`
/// Template resources: `screenshots://{id}`, `screenshots://{id}/ocr`
///
/// All reads delegate to the main Tauri app via the IPC bridge. When the
/// bridge is unavailable, reads return an MCP internal error.
use rmcp::model::{
    AnnotateAble, ListResourceTemplatesResult, ListResourcesResult, RawResource,
    RawResourceTemplate, ReadResourceResult, ResourceContents,
};
use rmcp::Error as McpError;
use serde_json::json;

use crate::bridge::AppBridge;

/// Static resources for `resources/list`.
pub fn list() -> ListResourcesResult {
    ListResourcesResult {
        next_cursor: None,
        resources: vec![
            RawResource {
                uri: "screenshots://recent".to_owned(),
                name: "Recent Screenshots".to_owned(),
                description: Some(
                    "List of recent screenshot captures with metadata".to_owned(),
                ),
                mime_type: Some("application/json".to_owned()),
                size: None,
            }
            .no_annotation(),
            RawResource {
                uri: "settings://current".to_owned(),
                name: "Current Settings".to_owned(),
                description: Some(
                    "Current application settings (capture, annotation, AI, UI)".to_owned(),
                ),
                mime_type: Some("application/json".to_owned()),
                size: None,
            }
            .no_annotation(),
        ],
    }
}

/// URI templates for `resources/templates/list`.
pub fn list_templates() -> ListResourceTemplatesResult {
    ListResourceTemplatesResult {
        next_cursor: None,
        resource_templates: vec![
            RawResourceTemplate {
                uri_template: "screenshots://{id}".to_owned(),
                name: "Screenshot".to_owned(),
                description: Some(
                    "Base64-encoded PNG image and metadata for a specific screenshot".to_owned(),
                ),
                mime_type: Some("image/png".to_owned()),
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "screenshots://{id}/ocr".to_owned(),
                name: "Screenshot OCR".to_owned(),
                description: Some(
                    "Cached or on-demand OCR text and regions for a specific screenshot"
                        .to_owned(),
                ),
                mime_type: Some("application/json".to_owned()),
            }
            .no_annotation(),
        ],
    }
}

/// Read a resource by URI.
pub async fn read(
    bridge: Option<&AppBridge>,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    if uri == "screenshots://recent" {
        return read_recent(bridge, uri).await;
    }
    if uri == "settings://current" {
        return read_settings(bridge, uri).await;
    }
    // screenshots://{id}/ocr — must check before screenshots://{id}
    if let Some(id) = uri
        .strip_prefix("screenshots://")
        .and_then(|s| s.strip_suffix("/ocr"))
    {
        if !id.is_empty() && !id.contains('/') {
            return read_ocr(bridge, uri, id).await;
        }
    }
    if let Some(id) = uri.strip_prefix("screenshots://") {
        if !id.is_empty() && !id.contains('/') {
            return read_screenshot(bridge, uri, id).await;
        }
    }
    Err(McpError::invalid_params(
        format!("unknown resource: {uri}"),
        None,
    ))
}

// --- resource handlers -------------------------------------------------------

async fn read_recent(
    bridge: Option<&AppBridge>,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    let value = bridge_call(bridge, "list_screenshots", json!({ "limit": 10 })).await?;
    Ok(ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
            uri: uri.to_owned(),
            mime_type: Some("application/json".to_owned()),
            text: value.to_string(),
        }],
    })
}

async fn read_screenshot(
    bridge: Option<&AppBridge>,
    uri: &str,
    id: &str,
) -> Result<ReadResourceResult, McpError> {
    let value = bridge_call(bridge, "get_screenshot", json!({ "id": id })).await?;

    let mut contents = vec![];
    if let Some(b64) = value.get("image_b64").and_then(|v| v.as_str()) {
        contents.push(ResourceContents::BlobResourceContents {
            uri: uri.to_owned(),
            mime_type: Some("image/png".to_owned()),
            blob: b64.to_owned(),
        });
    }
    let meta = json!({
        "id":        value.get("id"),
        "timestamp": value.get("timestamp"),
        "width":     value.get("width"),
        "height":    value.get("height"),
        "mode":      value.get("mode"),
    });
    contents.push(ResourceContents::TextResourceContents {
        uri: format!("{uri}#metadata"),
        mime_type: Some("application/json".to_owned()),
        text: meta.to_string(),
    });

    Ok(ReadResourceResult { contents })
}

async fn read_ocr(
    bridge: Option<&AppBridge>,
    uri: &str,
    id: &str,
) -> Result<ReadResourceResult, McpError> {
    let value = bridge_call(bridge, "run_ocr", json!({ "screenshot_id": id })).await?;
    Ok(ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
            uri: uri.to_owned(),
            mime_type: Some("application/json".to_owned()),
            text: value.to_string(),
        }],
    })
}

async fn read_settings(
    bridge: Option<&AppBridge>,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    let value = bridge_call(bridge, "get_settings", json!({})).await?;
    Ok(ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
            uri: uri.to_owned(),
            mime_type: Some("application/json".to_owned()),
            text: value.to_string(),
        }],
    })
}

// --- helpers -----------------------------------------------------------------

async fn bridge_call(
    bridge: Option<&AppBridge>,
    command: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, McpError> {
    let Some(bridge) = bridge else {
        return Err(McpError::internal_error(
            "Fotos app is not running. Start the Fotos application and try again.",
            None,
        ));
    };
    bridge.send_command(command, params).await.map_err(|e| {
        McpError::internal_error(format!("IPC error calling '{command}': {e}"), None)
    })
}
