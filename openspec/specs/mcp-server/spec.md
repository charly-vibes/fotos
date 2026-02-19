# Capability: MCP Server

## Purpose

The MCP server exposes Fotos functionality to AI agents (Claude Desktop, Cursor, etc.) via the Model Context Protocol. It runs as a standalone binary (`fotos-mcp`) that communicates over stdio using JSON-RPC 2.0. It delegates work to the main Tauri app via an IPC bridge and can also operate standalone for headless/CI use.

## Requirements

### Requirement: Standalone Binary

The MCP server SHALL be built as a standalone binary named `fotos-mcp`, separate from the main Tauri application binary. The binary SHALL be bundled alongside the main app in all distribution formats (Flatpak, AppImage, MSI/NSIS).

#### Scenario: Binary invocation by MCP host
- **WHEN** an MCP host (e.g., Claude Desktop) launches `fotos-mcp` as a child process
- **THEN** the binary SHALL start, initialize the MCP protocol handler, and begin accepting JSON-RPC 2.0 messages on stdin/stdout

#### Scenario: Flatpak invocation
- **WHEN** an MCP host invokes `flatpak run --command=fotos-mcp io.github.charly.fotos`
- **THEN** the `fotos-mcp` binary SHALL start within the Flatpak sandbox and operate identically to a direct invocation

### Requirement: Stdio Transport

The `fotos-mcp` binary SHALL communicate exclusively via stdio transport using the JSON-RPC 2.0 protocol. All MCP messages (requests, responses, notifications) SHALL be read from stdin and written to stdout. Diagnostic and log output MUST be written to stderr, never to stdout.

#### Scenario: JSON-RPC message exchange
- **WHEN** an MCP host sends a valid JSON-RPC 2.0 request on stdin
- **THEN** `fotos-mcp` SHALL parse the request and write a JSON-RPC 2.0 response to stdout

#### Scenario: Malformed input
- **WHEN** an MCP host sends invalid JSON on stdin
- **THEN** `fotos-mcp` SHALL respond with a JSON-RPC 2.0 error (code -32700, Parse error) and SHALL NOT crash

### Requirement: Server Identity

The MCP server SHALL advertise its identity during the `initialize` handshake with the following metadata:
- name: `fotos`
- version: the current build version (e.g., `0.1.0`)
- description: `AI-powered screenshot capture, annotation, and analysis`

#### Scenario: Initialize handshake
- **WHEN** an MCP host sends an `initialize` request
- **THEN** `fotos-mcp` SHALL respond with the server identity, supported protocol version, and the list of available capabilities (tools, resources, prompts)

---

### Requirement: IPC Communication with Main App

The `fotos-mcp` binary SHALL communicate with the main Tauri application via a local IPC channel. On Linux, the IPC channel SHALL use a Unix domain socket. On Windows, the IPC channel SHALL use a named pipe. The IPC protocol SHALL carry structured commands and responses between the two processes.

#### Scenario: Command delegation on Linux
- **WHEN** `fotos-mcp` receives an MCP tool call and the main Tauri app is running
- **THEN** `fotos-mcp` SHALL connect to the Unix domain socket, send the command, receive the result, and return it to the MCP host

#### Scenario: Command delegation on Windows
- **WHEN** `fotos-mcp` receives an MCP tool call and the main Tauri app is running on Windows
- **THEN** `fotos-mcp` SHALL connect to the named pipe, send the command, receive the result, and return it to the MCP host

### Requirement: Stateless Delegation

The `fotos-mcp` process SHALL be stateless. All screenshot storage, AI processing, and file operations MUST be delegated to the main Tauri application via the IPC bridge. The MCP server SHALL NOT maintain its own screenshot store or AI processing pipeline when operating in bridged mode.

#### Scenario: No local state retained between requests
- **WHEN** `fotos-mcp` completes a tool call and returns the result
- **THEN** no screenshot data, OCR results, or AI analysis results SHALL be cached in the `fotos-mcp` process memory

### Requirement: Main App Unavailable

When the main Tauri application is not running and `fotos-mcp` is not in standalone mode, tool calls that require the IPC bridge SHALL return an MCP error indicating the main application is unavailable.

#### Scenario: IPC connection failure
- **WHEN** `fotos-mcp` attempts to connect to the IPC channel and the main app is not running
- **THEN** `fotos-mcp` SHALL return a JSON-RPC 2.0 error response with a descriptive message indicating that the Fotos application is not running

---

### Requirement: Screenshot Capture Tool

The MCP server SHALL expose a `take_screenshot` tool that captures a screenshot of the desktop, a specific monitor, or a specific window. The tool SHALL accept the following optional parameters:
- `mode` (string, enum: `fullscreen`, `monitor`, `window`, default: `fullscreen`) -- the capture mode
- `monitor_index` (integer) -- the monitor index, used when mode is `monitor`
- `window_title` (string) -- a substring to match against window titles, used when mode is `window`
- `delay_ms` (integer, default: 0) -- delay in milliseconds before capture

#### Scenario: Fullscreen capture
- **WHEN** `take_screenshot` is called with `mode` set to `fullscreen`
- **THEN** the tool SHALL capture all monitors composited into a single image and return an image content block (base64 PNG) along with text metadata containing the screenshot ID, dimensions, and timestamp

#### Scenario: Single monitor capture
- **WHEN** `take_screenshot` is called with `mode` set to `monitor` and `monitor_index` set to `1`
- **THEN** the tool SHALL capture only the specified monitor and return the image content block with metadata

#### Scenario: Window capture
- **WHEN** `take_screenshot` is called with `mode` set to `window` and `window_title` set to `"Firefox"`
- **THEN** the tool SHALL find the first window whose title contains `"Firefox"` (case-insensitive) and capture it, returning the image content block with metadata

#### Scenario: Delayed capture
- **WHEN** `take_screenshot` is called with `delay_ms` set to `2000`
- **THEN** the tool SHALL wait 2000 milliseconds before performing the capture

#### Scenario: No matching window
- **WHEN** `take_screenshot` is called with `mode` set to `window` and no window matches `window_title`
- **THEN** the tool SHALL return an error indicating no matching window was found

---

### Requirement: OCR Text Extraction Tool

The MCP server SHALL expose an `ocr_screenshot` tool that extracts text from a screenshot using OCR. The tool SHALL accept the following parameters:
- `screenshot_id` (string, optional) -- ID of a previously captured screenshot; if omitted, a new fullscreen capture SHALL be taken first
- `language` (string, default: `eng`) -- the OCR language code (e.g., `eng`, `deu`, `jpn`)

The tool SHALL return a text content block containing the extracted text and structured regions with bounding boxes and confidence scores.

#### Scenario: OCR on existing screenshot
- **WHEN** `ocr_screenshot` is called with a valid `screenshot_id`
- **THEN** the tool SHALL run OCR on the referenced screenshot and return the extracted text along with per-word regions containing text, bounding box coordinates (x, y, w, h), and confidence values

#### Scenario: OCR with automatic capture
- **WHEN** `ocr_screenshot` is called without a `screenshot_id`
- **THEN** the tool SHALL first capture a new fullscreen screenshot, then run OCR on it, and return both the screenshot ID and the extracted text with regions

#### Scenario: Non-English OCR
- **WHEN** `ocr_screenshot` is called with `language` set to `deu`
- **THEN** the tool SHALL use the German language model for OCR processing

#### Scenario: Invalid screenshot ID
- **WHEN** `ocr_screenshot` is called with a `screenshot_id` that does not exist
- **THEN** the tool SHALL return an error indicating the screenshot was not found

---

### Requirement: Annotation Tool

The MCP server SHALL expose an `annotate_screenshot` tool that adds annotations to a screenshot. The tool SHALL accept the following parameters:
- `screenshot_id` (string, required) -- ID of the screenshot to annotate
- `annotations` (array, required) -- an array of annotation objects, each containing:
  - `type` (string, required, enum: `arrow`, `rect`, `ellipse`, `text`, `blur`) -- the annotation type
  - `x` (number) -- x-coordinate of the annotation origin
  - `y` (number) -- y-coordinate of the annotation origin
  - `width` (number) -- width of the annotation bounding box
  - `height` (number) -- height of the annotation bounding box
  - `points` (array of objects) -- control points for arrow and freehand types
  - `text` (string) -- text content for text annotations
  - `color` (string, default: `#FF0000`) -- stroke/fill color
  - `stroke_width` (number, default: 2) -- stroke width in pixels

The tool SHALL return the annotated image as a base64 PNG image content block.

#### Scenario: Add rectangle annotation
- **WHEN** `annotate_screenshot` is called with a rectangle annotation at coordinates (100, 100) with width 200 and height 150
- **THEN** the tool SHALL composite the rectangle onto the screenshot and return the annotated image as base64 PNG

#### Scenario: Add multiple annotations
- **WHEN** `annotate_screenshot` is called with an array containing an arrow, a text label, and a blur region
- **THEN** the tool SHALL render all annotations onto the screenshot in array order and return the composited image

#### Scenario: Invalid screenshot ID
- **WHEN** `annotate_screenshot` is called with a `screenshot_id` that does not exist
- **THEN** the tool SHALL return an error indicating the screenshot was not found

---

### Requirement: LLM Vision Analysis Tool

The MCP server SHALL expose an `analyze_screenshot` tool that sends a screenshot to an LLM vision model for analysis. The tool SHALL accept the following parameters:
- `screenshot_id` (string, optional) -- ID of a previously captured screenshot; if omitted, a new fullscreen capture SHALL be taken first
- `prompt` (string, default: `"Describe what you see in this screenshot"`) -- the analysis prompt sent to the LLM
- `provider` (string, enum: `claude`, `openai`, `gemini`, `ollama`, default: `claude`) -- the LLM provider to use

The tool SHALL return the LLM's text response along with metadata (model name, token usage).

#### Scenario: Analysis with default provider
- **WHEN** `analyze_screenshot` is called with a valid `screenshot_id` and a custom `prompt`
- **THEN** the tool SHALL send the screenshot image and prompt to the configured default provider (Claude) and return the LLM's text response

#### Scenario: Analysis with explicit provider
- **WHEN** `analyze_screenshot` is called with `provider` set to `ollama`
- **THEN** the tool SHALL route the request to the local Ollama instance using the configured model and URL

#### Scenario: Missing API key
- **WHEN** `analyze_screenshot` is called with a cloud provider and no API key is configured for that provider
- **THEN** the tool SHALL return an error indicating the API key is missing for the requested provider

#### Scenario: Automatic capture before analysis
- **WHEN** `analyze_screenshot` is called without a `screenshot_id`
- **THEN** the tool SHALL first capture a new fullscreen screenshot, then send it for LLM analysis

---

### Requirement: PII Auto-Redaction Tool

The MCP server SHALL expose an `auto_redact_pii` tool that detects and blurs personally identifiable information in a screenshot. The tool SHALL accept the following parameter:
- `screenshot_id` (string, required) -- ID of the screenshot to redact

The tool SHALL run OCR, apply PII pattern matching (email, phone, SSN, credit card, API keys, IP addresses), blur the detected regions, and return both the redacted image (base64 PNG) and a list of detected PII types with their locations.

#### Scenario: PII detected and blurred
- **WHEN** `auto_redact_pii` is called on a screenshot containing visible email addresses and phone numbers
- **THEN** the tool SHALL detect the PII regions, apply blur to each region, and return the redacted image along with an array of detections listing each PII type (e.g., `email`, `phone`) and bounding box coordinates

#### Scenario: No PII found
- **WHEN** `auto_redact_pii` is called on a screenshot with no detectable PII
- **THEN** the tool SHALL return the original image unchanged and an empty detections array

#### Scenario: Invalid screenshot ID
- **WHEN** `auto_redact_pii` is called with a `screenshot_id` that does not exist
- **THEN** the tool SHALL return an error indicating the screenshot was not found

---

### Requirement: List Screenshots Tool

The MCP server SHALL expose a `list_screenshots` tool that returns metadata for recent screenshots in the current session. The tool SHALL accept the following parameter:
- `limit` (integer, default: 10) -- the maximum number of screenshots to return

The tool SHALL return an array of screenshot metadata objects, each containing at minimum: screenshot ID, timestamp, dimensions, and capture mode.

#### Scenario: List with default limit
- **WHEN** `list_screenshots` is called without parameters and 15 screenshots exist in the session
- **THEN** the tool SHALL return metadata for the 10 most recent screenshots, ordered by timestamp descending

#### Scenario: List with custom limit
- **WHEN** `list_screenshots` is called with `limit` set to `3`
- **THEN** the tool SHALL return metadata for at most 3 screenshots

#### Scenario: No screenshots available
- **WHEN** `list_screenshots` is called and no screenshots have been captured in the session
- **THEN** the tool SHALL return an empty array

---

### Requirement: Recent Screenshots Resource

The MCP server SHALL expose a `screenshots://recent` resource that returns a list of recent screenshot metadata. This resource SHALL provide the same data as the `list_screenshots` tool with default parameters, accessible via the MCP resource read mechanism.

#### Scenario: Read recent screenshots resource
- **WHEN** an MCP host reads the `screenshots://recent` resource
- **THEN** the server SHALL return a list of recent screenshot metadata entries including IDs, timestamps, dimensions, and capture modes

### Requirement: Individual Screenshot Resource

The MCP server SHALL expose a `screenshots://{id}` resource that returns the image and metadata for a specific screenshot, identified by its UUID.

#### Scenario: Read existing screenshot
- **WHEN** an MCP host reads `screenshots://abc-123` where `abc-123` is a valid screenshot ID
- **THEN** the server SHALL return the screenshot image (base64 PNG) and its metadata (timestamp, dimensions, capture mode)

#### Scenario: Read nonexistent screenshot
- **WHEN** an MCP host reads `screenshots://nonexistent-id`
- **THEN** the server SHALL return a resource-not-found error

### Requirement: Screenshot OCR Resource

The MCP server SHALL expose a `screenshots://{id}/ocr` resource that returns cached OCR results for a specific screenshot. If OCR has not been run on the screenshot, the server SHALL run OCR on demand and return the results.

#### Scenario: Read cached OCR results
- **WHEN** an MCP host reads `screenshots://abc-123/ocr` and OCR was previously run on screenshot `abc-123`
- **THEN** the server SHALL return the cached extracted text and regions

#### Scenario: Read OCR results without prior OCR
- **WHEN** an MCP host reads `screenshots://abc-123/ocr` and OCR has not been run on screenshot `abc-123`
- **THEN** the server SHALL run OCR on the screenshot, cache the results, and return the extracted text and regions

### Requirement: Current Settings Resource

The MCP server SHALL expose a `settings://current` resource that returns the current application settings, including capture preferences, annotation defaults, AI configuration, and UI preferences.

#### Scenario: Read current settings
- **WHEN** an MCP host reads the `settings://current` resource
- **THEN** the server SHALL return the current settings as a JSON object matching the settings schema (capture, annotation, ai, ui sections)

---

### Requirement: Describe UI Prompt

The MCP server SHALL expose a `describe_ui` prompt that generates a request to describe the UI elements visible in a screenshot. The prompt SHALL require a `screenshot_id` argument.

#### Scenario: Generate describe UI prompt
- **WHEN** an MCP host requests the `describe_ui` prompt with `screenshot_id` set to `abc-123`
- **THEN** the server SHALL return a prompt message containing the screenshot image and the instruction to describe the UI elements visible in the screenshot

### Requirement: Extract Code Prompt

The MCP server SHALL expose an `extract_code` prompt that generates a request to extract all code visible in a screenshot. The prompt SHALL require a `screenshot_id` argument.

#### Scenario: Generate extract code prompt
- **WHEN** an MCP host requests the `extract_code` prompt with `screenshot_id` set to `abc-123`
- **THEN** the server SHALL return a prompt message containing the screenshot image and the instruction to extract all visible code, returning only properly formatted code

### Requirement: Generate Bug Report Prompt

The MCP server SHALL expose a `generate_bug_report` prompt that generates a request to create a bug report from a screenshot showing an error. The prompt SHALL require a `screenshot_id` argument and accept an optional `context` argument for additional context about the bug.

#### Scenario: Generate bug report prompt without context
- **WHEN** an MCP host requests the `generate_bug_report` prompt with `screenshot_id` set to `abc-123` and no `context`
- **THEN** the server SHALL return a prompt message containing the screenshot image and the instruction to analyze the error and generate a bug report with summary, steps to reproduce, expected vs. actual behavior, and severity assessment

#### Scenario: Generate bug report prompt with context
- **WHEN** an MCP host requests the `generate_bug_report` prompt with `screenshot_id` set to `abc-123` and `context` set to `"This crash happens after clicking the submit button"`
- **THEN** the server SHALL return a prompt message containing the screenshot image, the additional context, and the bug report generation instruction

### Requirement: Accessibility Audit Prompt

The MCP server SHALL expose an `accessibility_audit` prompt that generates a request to audit a UI screenshot for accessibility issues. The prompt SHALL require a `screenshot_id` argument.

#### Scenario: Generate accessibility audit prompt
- **WHEN** an MCP host requests the `accessibility_audit` prompt with `screenshot_id` set to `abc-123`
- **THEN** the server SHALL return a prompt message containing the screenshot image and the instruction to audit the UI for accessibility issues

---

### Requirement: Standalone Operation

The `fotos-mcp` binary SHALL support a standalone mode of operation where it captures screenshots and performs AI processing without requiring the main Tauri application to be running. In standalone mode, the binary SHALL use platform capture APIs directly (xcap for X11/Windows, xdg-desktop-portal for Wayland) and manage its own temporary screenshot storage.

#### Scenario: Capture in standalone mode
- **WHEN** `fotos-mcp` receives a `take_screenshot` tool call and the main Tauri app is not running
- **THEN** `fotos-mcp` SHALL capture the screenshot directly using platform APIs, store it in temporary memory, and return the image content block with metadata

#### Scenario: Headless/CI usage
- **WHEN** `fotos-mcp` is invoked in a headless CI environment without a display server
- **THEN** `fotos-mcp` SHALL return an error indicating that no display server is available for screenshot capture

### Requirement: Standalone Mode Limitations

When operating in standalone mode, `fotos-mcp` SHALL NOT provide annotation persistence, settings management via the app UI, or undo/redo functionality. These features MUST only be available when the main Tauri application is running and the IPC bridge is connected.

#### Scenario: Standalone tool availability
- **WHEN** `fotos-mcp` is operating in standalone mode
- **THEN** `take_screenshot`, `ocr_screenshot`, `analyze_screenshot`, `auto_redact_pii`, and `list_screenshots` SHALL be available, while `annotate_screenshot` SHALL function using in-memory rendering without persisting annotations to the main app

#### Scenario: Settings resource in standalone mode
- **WHEN** an MCP host reads `settings://current` while `fotos-mcp` is in standalone mode
- **THEN** the server SHALL return default settings values since the main app settings store is unavailable
