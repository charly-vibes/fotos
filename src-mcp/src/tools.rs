/// MCP tool definitions and dispatch.
///
/// Tools delegate to the main Tauri app via the IPC bridge. When the bridge
/// is unavailable (app not running), calls return an error content block
/// rather than an MCP protocol error, per the spec.
use std::sync::Arc;

use rmcp::model::{CallToolResult, Content, ListToolsResult, Tool};
use rmcp::Error as McpError;
use serde_json::{Map, Value};

use crate::bridge::AppBridge;

/// Returns all 6 tool definitions for `tools/list`.
pub fn list() -> ListToolsResult {
    ListToolsResult {
        next_cursor: None,
        tools: vec![
            Tool::new(
                "take_screenshot",
                "Capture a screenshot of the desktop, a specific monitor, or a specific window.",
                schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "mode": {
                            "type": "string",
                            "enum": ["fullscreen", "monitor", "window"],
                            "default": "fullscreen",
                            "description": "The capture mode"
                        },
                        "monitor_index": {
                            "type": "integer",
                            "description": "Monitor index (used when mode is 'monitor')"
                        },
                        "window_title": {
                            "type": "string",
                            "description": "Substring to match against window titles (used when mode is 'window')"
                        },
                        "delay_ms": {
                            "type": "integer",
                            "default": 0,
                            "description": "Delay in milliseconds before capture"
                        }
                    }
                })),
            ),
            Tool::new(
                "ocr_screenshot",
                "Extract text from a screenshot using OCR.",
                schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "screenshot_id": {
                            "type": "string",
                            "description": "ID of a previously captured screenshot; if omitted, a new fullscreen capture is taken first"
                        },
                        "language": {
                            "type": "string",
                            "default": "eng",
                            "description": "OCR language code (e.g. 'eng', 'deu', 'jpn')"
                        }
                    }
                })),
            ),
            Tool::new(
                "annotate_screenshot",
                "Add annotations (rectangles, arrows, text, blur regions) to a screenshot and return the composited image.",
                schema(serde_json::json!({
                    "type": "object",
                    "required": ["screenshot_id", "annotations"],
                    "properties": {
                        "screenshot_id": {
                            "type": "string",
                            "description": "ID of the screenshot to annotate"
                        },
                        "annotations": {
                            "type": "array",
                            "description": "Array of annotation objects",
                            "items": {
                                "type": "object",
                                "required": ["type"],
                                "properties": {
                                    "type": {
                                        "type": "string",
                                        "description": "Annotation type (rect, arrow, text, step, blur, freehand)"
                                    },
                                    "x": { "type": "number" },
                                    "y": { "type": "number" },
                                    "width": { "type": "number" },
                                    "height": { "type": "number" },
                                    "points": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "properties": {
                                                "x": { "type": "number" },
                                                "y": { "type": "number" }
                                            }
                                        }
                                    },
                                    "text": { "type": "string" },
                                    "strokeColor": {
                                        "type": "string",
                                        "default": "#FF0000"
                                    },
                                    "strokeWidth": {
                                        "type": "number",
                                        "default": 2
                                    }
                                }
                            }
                        }
                    }
                })),
            ),
            Tool::new(
                "analyze_screenshot",
                "Send a screenshot to an LLM vision model for analysis.",
                schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "screenshot_id": {
                            "type": "string",
                            "description": "ID of a previously captured screenshot; if omitted, a new fullscreen capture is taken first"
                        },
                        "prompt": {
                            "type": "string",
                            "default": "Describe what you see in this screenshot in detail.",
                            "description": "The analysis prompt sent to the LLM"
                        },
                        "provider": {
                            "type": "string",
                            "enum": ["claude", "openai", "gemini", "ollama"],
                            "default": "claude",
                            "description": "The LLM provider to use"
                        }
                    }
                })),
            ),
            Tool::new(
                "auto_redact_pii",
                "Detect and blur personally identifiable information (email, phone, SSN, credit card, etc.) in a screenshot.",
                schema(serde_json::json!({
                    "type": "object",
                    "required": ["screenshot_id"],
                    "properties": {
                        "screenshot_id": {
                            "type": "string",
                            "description": "ID of the screenshot to redact"
                        }
                    }
                })),
            ),
            Tool::new(
                "list_screenshots",
                "Return metadata for recent screenshots in the current session.",
                schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "default": 10,
                            "description": "Maximum number of screenshots to return"
                        }
                    }
                })),
            ),
        ],
    }
}

/// Dispatch a `tools/call` request.
pub async fn call(
    bridge: Option<&AppBridge>,
    name: &str,
    args: Option<&Map<String, Value>>,
) -> Result<CallToolResult, McpError> {
    let empty = Map::new();
    let args = args.unwrap_or(&empty);

    match name {
        "take_screenshot"
        | "ocr_screenshot"
        | "annotate_screenshot"
        | "analyze_screenshot"
        | "auto_redact_pii"
        | "list_screenshots" => call_via_bridge(bridge, name, args).await,
        _ => Err(McpError::invalid_params(
            format!("unknown tool: {name}"),
            None,
        )),
    }
}

// --- internals ---------------------------------------------------------------

async fn call_via_bridge(
    bridge: Option<&AppBridge>,
    command: &str,
    params: &Map<String, Value>,
) -> Result<CallToolResult, McpError> {
    let Some(bridge) = bridge else {
        return Ok(CallToolResult::error(vec![Content::text(
            "Fotos app is not running. Start the Fotos application and try again.",
        )]));
    };

    match bridge
        .send_command(command, Value::Object(params.clone()))
        .await
    {
        Ok(value) => Ok(CallToolResult::success(format_result(command, value))),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "IPC error: {e}"
        ))])),
    }
}

/// Format the IPC response into MCP content blocks.
fn format_result(command: &str, value: Value) -> Vec<Content> {
    match command {
        "take_screenshot" => {
            // { id, image_b64, width, height, timestamp, mode }
            let meta = serde_json::json!({
                "id":        value.get("id"),
                "width":     value.get("width"),
                "height":    value.get("height"),
                "timestamp": value.get("timestamp"),
                "mode":      value.get("mode"),
            });
            let mut out = vec![Content::text(meta.to_string())];
            if let Some(b64) = value.get("image_b64").and_then(|v| v.as_str()) {
                out.push(Content::image(b64.to_owned(), "image/png"));
            }
            out
        }
        "annotate_screenshot" => {
            // { image_b64 }
            if let Some(b64) = value.get("image_b64").and_then(|v| v.as_str()) {
                vec![Content::image(b64.to_owned(), "image/png")]
            } else {
                vec![Content::text(value.to_string())]
            }
        }
        "auto_redact_pii" => {
            // { image_b64, detections: [{type, x, y, w, h}] }
            let mut out = vec![];
            if let Some(b64) = value.get("image_b64").and_then(|v| v.as_str()) {
                out.push(Content::image(b64.to_owned(), "image/png"));
            }
            let detections = value.get("detections").cloned().unwrap_or(Value::Array(vec![]));
            out.push(Content::text(detections.to_string()));
            out
        }
        _ => vec![Content::text(value.to_string())],
    }
}

/// Build a JSON Schema `Arc<JsonObject>` from a `serde_json::Value`.
fn schema(value: Value) -> Arc<Map<String, Value>> {
    match value {
        Value::Object(map) => Arc::new(map),
        _ => Arc::new(Map::new()),
    }
}
