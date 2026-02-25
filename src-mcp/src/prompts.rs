/// MCP prompt template definitions and handlers.
///
/// The 4 prompts accept a `screenshot_id` argument and return a messages array
/// ready to send to an LLM. Each prompt embeds the screenshot via its resource
/// URI (`screenshots://{id}`) — the MCP client resolves the resource.
use rmcp::model::{
    GetPromptRequestParam, GetPromptResult, ListPromptsResult, Prompt, PromptArgument,
    PromptMessage, PromptMessageRole,
};
use rmcp::Error as McpError;

/// Returns all 4 prompt definitions for `prompts/list`.
pub fn list() -> ListPromptsResult {
    ListPromptsResult {
        next_cursor: None,
        prompts: vec![
            Prompt::new(
                "describe_ui",
                Some("Describe the UI elements visible in a screenshot"),
                Some(vec![screenshot_id_arg(true)]),
            ),
            Prompt::new(
                "extract_code",
                Some("Extract all code visible in a screenshot"),
                Some(vec![screenshot_id_arg(true)]),
            ),
            Prompt::new(
                "generate_bug_report",
                Some("Generate a bug report from a screenshot showing an error"),
                Some(vec![
                    screenshot_id_arg(true),
                    PromptArgument {
                        name: "context".to_string(),
                        description: Some(
                            "Optional additional context about the bug (e.g. steps to reproduce)"
                                .to_string(),
                        ),
                        required: Some(false),
                    },
                ]),
            ),
            Prompt::new(
                "accessibility_audit",
                Some("Audit a UI screenshot for accessibility issues"),
                Some(vec![screenshot_id_arg(true)]),
            ),
        ],
    }
}

/// Dispatches a `prompts/get` request to the matching prompt handler.
pub fn get(request: GetPromptRequestParam) -> Result<GetPromptResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let screenshot_id = args
        .get("screenshot_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("missing required argument: screenshot_id", None))?
        .to_owned();

    match request.name.as_ref() {
        "describe_ui" => Ok(describe_ui(screenshot_id)),
        "extract_code" => Ok(extract_code(screenshot_id)),
        "generate_bug_report" => {
            let context = args
                .get("context")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            Ok(generate_bug_report(screenshot_id, context))
        }
        "accessibility_audit" => Ok(accessibility_audit(screenshot_id)),
        name => Err(McpError::invalid_params(
            format!("unknown prompt: {name}"),
            None,
        )),
    }
}

// --- helpers -----------------------------------------------------------------

fn screenshot_id_arg(required: bool) -> PromptArgument {
    PromptArgument {
        name: "screenshot_id".to_string(),
        description: Some("ID of the screenshot to analyse".to_string()),
        required: Some(required),
    }
}

/// Embed the screenshot as a resource reference (resolved by the MCP client
/// via `resources/read screenshots://{id}`).
fn screenshot_resource_message(screenshot_id: &str) -> PromptMessage {
    PromptMessage::new_resource(
        PromptMessageRole::User,
        format!("screenshots://{screenshot_id}"),
        "image/png".to_string(),
        None,
        None,
    )
}

// --- prompt builders ---------------------------------------------------------

fn describe_ui(screenshot_id: String) -> GetPromptResult {
    GetPromptResult {
        description: Some("Describe UI elements in the screenshot".to_string()),
        messages: vec![
            screenshot_resource_message(&screenshot_id),
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Describe all UI elements visible in this screenshot in detail. \
                 Include the overall layout, interactive elements (buttons, inputs, links), \
                 text content, colours, icons, and visual hierarchy.",
            ),
        ],
    }
}

fn extract_code(screenshot_id: String) -> GetPromptResult {
    GetPromptResult {
        description: Some("Extract code visible in the screenshot".to_string()),
        messages: vec![
            screenshot_resource_message(&screenshot_id),
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Extract all code visible in this screenshot. \
                 Return only the code, properly formatted, with the correct language identifier \
                 in a fenced code block. If multiple snippets are visible, return each in its \
                 own fenced block with the appropriate language.",
            ),
        ],
    }
}

fn generate_bug_report(screenshot_id: String, context: Option<String>) -> GetPromptResult {
    let instruction = if let Some(ctx) = context {
        format!(
            "Additional context: {ctx}\n\n\
             Analyse the error shown in this screenshot and generate a detailed bug report \
             containing: (1) a concise summary, (2) steps to reproduce, \
             (3) expected vs actual behaviour, and (4) a severity assessment."
        )
    } else {
        "Analyse the error shown in this screenshot and generate a detailed bug report \
         containing: (1) a concise summary, (2) steps to reproduce, \
         (3) expected vs actual behaviour, and (4) a severity assessment."
            .to_string()
    };

    GetPromptResult {
        description: Some("Generate a bug report from the screenshot".to_string()),
        messages: vec![
            screenshot_resource_message(&screenshot_id),
            PromptMessage::new_text(PromptMessageRole::User, instruction),
        ],
    }
}

fn accessibility_audit(screenshot_id: String) -> GetPromptResult {
    GetPromptResult {
        description: Some("Audit the screenshot for accessibility issues".to_string()),
        messages: vec![
            screenshot_resource_message(&screenshot_id),
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Audit the UI shown in this screenshot for accessibility issues. \
                 Check for: colour contrast, missing alt text indicators, font size legibility, \
                 touch target sizes, keyboard navigation order hints, and WCAG 2.1 AA compliance. \
                 List each issue with its location, the relevant guideline, and a suggested fix.",
            ),
        ],
    }
}
