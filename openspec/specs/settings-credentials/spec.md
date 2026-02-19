# Capability: Settings & Credentials

User preferences and API key management for Fotos (`io.github.charly.fotos`).

## Purpose

Preferences are persisted via `tauri-plugin-store`. API keys are stored in the OS keychain via the `keyring` crate with service name `fotos`.

## Requirements

### Requirement: Preference Persistence

The application SHALL persist all user preferences using `tauri-plugin-store` so that settings survive application restarts.

#### Scenario: Settings round-trip across restarts
- **WHEN** the user modifies any preference and restarts the application
- **THEN** the previously saved value SHALL be loaded and applied on startup

#### Scenario: First launch defaults
- **WHEN** the application launches for the first time and no store file exists
- **THEN** every preference SHALL assume its documented default value

### Requirement: Preference Schema

The preference store SHALL contain exactly four top-level sections: `capture`, `annotation`, `ai`, and `ui`. Each section SHALL conform to the schema defined in the following subsection requirements.

#### Scenario: Unknown keys ignored
- **WHEN** the store file contains a key not defined in the schema
- **THEN** the application SHALL ignore the unknown key without error

#### Scenario: Missing keys filled with defaults
- **WHEN** a schema-defined key is absent from the store file
- **THEN** the application SHALL treat the key as having its documented default value

---

### Requirement: Capture Preferences

The `capture` section SHALL support the following keys with their types and defaults:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `defaultMode` | string enum: `region`, `fullscreen`, `monitor`, `window` | `region` | Default capture mode |
| `includeMouseCursor` | boolean | `false` | Whether to include the mouse cursor in captures |
| `delayMs` | integer >= 0 | `0` | Delay in milliseconds before capture begins |
| `saveDirectory` | string (filesystem path) | `~/Pictures/Fotos` | Default directory for saved screenshots |
| `defaultFormat` | string enum: `png`, `jpg`, `webp` | `png` | Default image format for saving |
| `jpegQuality` | integer 1..100 | `90` | JPEG quality when format is `jpg` |
| `copyToClipboardAfterCapture` | boolean | `true` | Whether to copy the screenshot to the clipboard immediately after capture |

#### Scenario: Default capture mode applied
- **WHEN** the user invokes a capture without explicitly choosing a mode
- **THEN** the capture mode specified by `defaultMode` SHALL be used

#### Scenario: Capture delay respected
- **WHEN** `delayMs` is set to a value greater than zero
- **THEN** the application SHALL wait the specified number of milliseconds before performing the capture

#### Scenario: Copy to clipboard after capture
- **WHEN** `copyToClipboardAfterCapture` is `true` and the user takes a screenshot
- **THEN** the captured image SHALL be automatically copied to the system clipboard

#### Scenario: JPEG quality applied
- **WHEN** the user saves an image with format `jpg`
- **THEN** the image SHALL be encoded at the quality level specified by `jpegQuality`

---

### Requirement: Annotation Preferences

The `annotation` section SHALL support the following keys with their types and defaults:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `defaultStrokeColor` | string (CSS hex color) | `#FF0000` | Default stroke color for annotation tools |
| `defaultStrokeWidth` | number > 0 | `2` | Default stroke width in pixels |
| `defaultFontSize` | number > 0 | `16` | Default font size for text annotations |
| `defaultFontFamily` | string | `sans-serif` | Default font family for text annotations |
| `stepNumberColor` | string (CSS hex color) | `#FF0000` | Color for step-number annotation circles and text |
| `stepNumberSize` | number > 0 | `24` | Size of step-number annotations in pixels |
| `blurRadius` | number > 0 | `10` | Default blur radius for the blur/pixelate tool |

#### Scenario: Stroke color applied to new annotations
- **WHEN** the user creates a new annotation without explicitly choosing a color
- **THEN** the annotation SHALL use the `defaultStrokeColor` value

#### Scenario: Stroke width applied to new annotations
- **WHEN** the user creates a new shape annotation without explicitly choosing a width
- **THEN** the annotation SHALL use the `defaultStrokeWidth` value

#### Scenario: Font settings applied to text annotations
- **WHEN** the user creates a new text annotation without explicitly choosing font settings
- **THEN** the annotation SHALL use the `defaultFontSize` and `defaultFontFamily` values

#### Scenario: Step number styling applied
- **WHEN** the user places a step-number annotation
- **THEN** the step number SHALL use the `stepNumberColor` and `stepNumberSize` values as defaults

#### Scenario: Blur radius applied
- **WHEN** the user creates a blur region without explicitly choosing a radius
- **THEN** the blur SHALL use the `blurRadius` value

---

### Requirement: AI Preferences

The `ai` section SHALL support the following keys with their types and defaults:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ocrLanguage` | string (Tesseract language code) | `eng` | Language for OCR text extraction |
| `defaultLlmProvider` | string enum: `claude`, `openai`, `gemini`, `ollama` | `claude` | Default LLM provider for vision analysis |
| `ollamaUrl` | string (URL) | `http://localhost:11434` | Base URL for the Ollama API |
| `ollamaModel` | string | `llava:7b` | Model name for Ollama vision requests |
| `claudeModel` | string | `claude-sonnet-4-20250514` | Model name for Anthropic Claude API requests |
| `openaiModel` | string | `gpt-4o` | Model name for OpenAI API requests |
| `geminiModel` | string | `gemini-2.0-flash` | Model name for Google Gemini API requests |

#### Scenario: OCR language applied
- **WHEN** the user invokes OCR without explicitly choosing a language
- **THEN** the OCR engine SHALL use the language specified by `ocrLanguage`

#### Scenario: Default LLM provider selected
- **WHEN** the user invokes LLM vision analysis without explicitly choosing a provider
- **THEN** the application SHALL use the provider specified by `defaultLlmProvider`

#### Scenario: Ollama endpoint configuration
- **WHEN** the user has set `ollamaUrl` and selects Ollama as the provider
- **THEN** the application SHALL send requests to the configured URL using the configured `ollamaModel`

#### Scenario: Cloud model override
- **WHEN** the user changes `claudeModel`, `openaiModel`, or `geminiModel`
- **THEN** subsequent API requests to the corresponding provider SHALL use the updated model name

---

### Requirement: UI Preferences

The `ui` section SHALL support the following keys with their types and defaults:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `theme` | string enum: `system`, `light`, `dark` | `system` | Application color theme |
| `showAiPanel` | boolean | `true` | Whether the AI results panel is visible |
| `showStatusBar` | boolean | `true` | Whether the bottom status bar is visible |

#### Scenario: System theme followed
- **WHEN** `theme` is set to `system`
- **THEN** the application SHALL follow the operating system's light/dark mode preference

#### Scenario: Explicit light theme
- **WHEN** `theme` is set to `light`
- **THEN** the application SHALL use the light color scheme regardless of the OS setting

#### Scenario: Explicit dark theme
- **WHEN** `theme` is set to `dark`
- **THEN** the application SHALL use the dark color scheme regardless of the OS setting

#### Scenario: AI panel visibility
- **WHEN** `showAiPanel` is `false`
- **THEN** the AI results sidebar SHALL be hidden from the UI

#### Scenario: Status bar visibility
- **WHEN** `showStatusBar` is `false`
- **THEN** the bottom status bar SHALL be hidden from the UI

---

### Requirement: OS Keychain Storage

All API keys SHALL be stored in the operating system keychain using the `keyring` crate with service name `fotos`. The following accounts SHALL be used:

| Provider | Service | Account |
|----------|---------|---------|
| Anthropic | `fotos` | `anthropic-api-key` |
| OpenAI | `fotos` | `openai-api-key` |
| Google | `fotos` | `google-api-key` |

#### Scenario: Store API key
- **WHEN** the user provides an API key for a supported provider via the settings UI
- **THEN** the application SHALL store the key in the OS keychain under service `fotos` with the corresponding account name

#### Scenario: Retrieve API key for LLM request
- **WHEN** the application needs to make an API request to a cloud LLM provider
- **THEN** it SHALL retrieve the API key from the OS keychain using service `fotos` and the provider's account name

#### Scenario: Delete API key
- **WHEN** the user removes an API key for a provider via the settings UI
- **THEN** the application SHALL delete the key from the OS keychain

#### Scenario: Missing API key
- **WHEN** the user attempts to use a cloud LLM provider and no API key is stored for that provider
- **THEN** the application SHALL display an error indicating that an API key is required and guide the user to the settings

---

### Requirement: API Keys Never in Config or LocalStorage

API keys MUST NOT be stored in configuration files, the `tauri-plugin-store` preference file, browser `localStorage`, or any other plaintext location. The OS keychain SHALL be the sole persistent storage for API keys.

#### Scenario: Preference store does not contain API keys
- **WHEN** the preference store file is inspected on disk
- **THEN** it SHALL NOT contain any API key values for any provider

#### Scenario: LocalStorage does not contain API keys
- **WHEN** the webview's `localStorage` is inspected
- **THEN** it SHALL NOT contain any API key values for any provider

#### Scenario: API keys not logged
- **WHEN** the application logs debug or error messages
- **THEN** API key values MUST NOT appear in log output
