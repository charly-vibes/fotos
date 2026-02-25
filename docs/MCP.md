# AI Agent Integration (MCP)

Fotos includes a Model Context Protocol (MCP) server named `fotos-mcp`. This server allows AI agents like Claude Desktop, Cursor, or Claude Code to programmatically capture, annotate, and analyze screenshots using Fotos.

## Current Implementation Status

`fotos-mcp` is currently under active development.

- ✅ **Prompts**: 4 prompt templates (`describe_ui`, `extract_code`, `generate_bug_report`, `accessibility_audit`) are fully implemented and ready for use.
- 🚧 **Tools**: `take_screenshot`, `ocr_screenshot`, etc. are currently **stubs** and will be implemented in the next phase (`fotos-0j0`).
- 🚧 **Resources**: `screenshots://`, `settings://`, etc. are currently **stubs** and will be implemented in the next phase (`fotos-rsw`).

## Usage with AI Agents

### 1. Claude Desktop Configuration

To use Fotos with Claude Desktop, add the following to your `claude_desktop_config.json`:

#### Direct Execution (from repo)
```json
{
  "mcpServers": {
    "fotos": {
      "command": "/path/to/fotos/target/release/fotos-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

#### Flatpak Execution
```json
{
  "mcpServers": {
    "fotos": {
      "command": "flatpak",
      "args": ["run", "--command=fotos-mcp", "io.github.charly.fotos"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 2. Available Prompts

| Prompt | Arguments | Description |
|---|---|---|
| `describe_ui` | `screenshot_id` | Generates a detailed description of the UI elements visible in the screenshot. |
| `extract_code` | `screenshot_id` | Extracts all visible code blocks from the screenshot and returns them in a formatted way. |
| `generate_bug_report` | `screenshot_id`, `context` (optional) | Analyzes the screenshot for errors and generates a structured bug report. |
| `accessibility_audit` | `screenshot_id` | Audits the UI for common accessibility issues (contrast, alt text, etc.). |

### 3. Usage Example (Claude)

Once the server is configured, you can tell Claude:

- "Describe the UI in screenshot `abc-123` using the `describe_ui` prompt."
- "Create a bug report for screenshot `xyz-789` using the `generate_bug_report` prompt."

## Use Case Guides

These guides demonstrate how to combine Fotos MCP tools and prompts for common AI-assisted workflows.

### 1. Autonomous Debugging
Help your AI agent "see" and fix code errors in real-time.

1. **Capture**: Tell the agent: "Take a screenshot of my terminal showing the failing test output."
2. **Analyze**: The agent uses `ocr_screenshot` to read the error message.
3. **Report**: Use the `generate_bug_report` prompt: "Analyze the error in screenshot `xyz` and generate a bug report."
4. **Fix**: The agent now has the full visual context (including logs that might not have been copied to the clipboard) to propose a fix.

### 2. Automated UI/UX & Accessibility Auditing
Ensure your UI meets design standards and WCAG accessibility guidelines.

1. **Capture**: "Capture a screenshot of the new login screen."
2. **Audit**: Use the `accessibility_audit` prompt: "Audit the login screen screenshot for accessibility issues."
3. **Feedback**: The agent will identify issues like low contrast, missing alt-text indicators, and touch target sizes.
4. **Iterate**: The agent can then suggest CSS or HTML changes to fix the identified problems.

### 3. Privacy-Preserving Technical Support
Safely share logs or UI states with AI agents without leaking secrets.

1. **Capture**: "Capture a screenshot of my current AWS console window."
2. **Redact**: Tell the agent: "Run `auto_redact_pii` on the screenshot to blur all sensitive information."
3. **Confirm**: The agent will return the redacted image and a list of blurred items (e.g., "email", "api_key").
4. **Assist**: You can now safely ask the agent for help with the console configuration, knowing your secrets are hidden.

### 4. Legacy System "API-ification"
Scrape data from old applications that don't have modern export functions.

1. **Capture**: "Take a screenshot of the legacy CRM application window."
2. **Extract**: "Run `ocr_screenshot` on the CRM screenshot."
3. **Transform**: Ask the agent: "Convert the OCR results from the CRM screenshot into a structured JSON list of customers and their last contact dates."
4. **Utilize**: The agent can then process this data or upload it to a modern database.

### 5. Automated Documentation & Tutorial Generation
Create professional, annotated step-by-step guides for users.

1. **Capture**: "Capture a series of screenshots as I perform a password reset."
2. **Annotate**: Tell the agent: "Use `annotate_screenshot` to add red boxes around the 'Reset Password' button and the email input field in these screenshots."
3. **Number**: "Add step-number annotations (1, 2, 3) to the key actions in each image."
4. **Draft**: "Generate a Markdown tutorial using these annotated screenshots to explain the password reset process."

## Architecture

`fotos-mcp` is a stateless binary that communicates with the main Fotos application via an IPC bridge:
- **Linux**: Unix domain socket (`$XDG_RUNTIME_DIR/fotos-ipc.sock` or `/tmp/fotos-ipc.sock`)
- **Windows**: Named pipe (`\.\pipe\fotos-ipc`)

When the main app is running, `fotos-mcp` delegates all capture and processing tasks to it. If the app is not running, `fotos-mcp` can potentially operate in a standalone mode for headless capture (feature in progress).

## Troubleshooting

- **Logs**: `fotos-mcp` logs to `stderr`. You can check the logs in your MCP client's console.
- **IPC Connection**: Ensure the main Fotos application is running if you need to perform actions that require its state or GUI.
- **Flatpak Permissions**: If running inside Flatpak, ensure the MCP client has permission to execute `flatpak run`.
