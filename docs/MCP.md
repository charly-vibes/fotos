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

## Architecture

`fotos-mcp` is a stateless binary that communicates with the main Fotos application via an IPC bridge:
- **Linux**: Unix domain socket (`$XDG_RUNTIME_DIR/fotos-ipc.sock` or `/tmp/fotos-ipc.sock`)
- **Windows**: Named pipe (`\.\pipe\fotos-ipc`)

When the main app is running, `fotos-mcp` delegates all capture and processing tasks to it. If the app is not running, `fotos-mcp` can potentially operate in a standalone mode for headless capture (feature in progress).

## Troubleshooting

- **Logs**: `fotos-mcp` logs to `stderr`. You can check the logs in your MCP client's console.
- **IPC Connection**: Ensure the main Fotos application is running if you need to perform actions that require its state or GUI.
- **Flatpak Permissions**: If running inside Flatpak, ensure the MCP client has permission to execute `flatpak run`.
