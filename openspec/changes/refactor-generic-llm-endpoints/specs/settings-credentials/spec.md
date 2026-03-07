## MODIFIED Requirements

### Requirement: AI Preferences

The `ai` section SHALL support the following keys with their types and defaults:

| Key | Type | Default | Description |
|---|---|---|---|
| `ocrLanguage` | string (Tesseract language code) | `eng` | Language for OCR text extraction |
| `defaultLlmProvider` | string | `claude` | Default provider: `"claude"`, `"gemini"`, or `"endpoint:{id}"` |
| `claudeModel` | string | `claude-sonnet-4-20250514` | Model name for Anthropic Claude API requests |
| `geminiModel` | string | `gemini-2.0-flash` | Model name for Google Gemini API requests |
| `endpoints` | array of `LlmEndpoint` | see below | User-defined OpenAI-compatible endpoints |

The following keys are **removed**: `openaiModel`, `ollamaUrl`, `ollamaModel`.

**`LlmEndpoint` object schema:**

| Field | Type | Description |
|---|---|---|
| `id` | string (8-char hex, truncated v4 UUID) | Unique identifier; used to key the keychain account |
| `name` | string | Display name shown in the provider selector |
| `base_url` | string (URL including API version path) | Base URL of the OpenAI-compatible server. MUST include the version path segment, e.g. `https://api.openai.com/v1`. The system appends `/chat/completions` to form the request URL. |
| `model` | string | Model name passed in the request body |

**Default `endpoints` list (first-run and after reset):**

```json
[
  { "id": "openai",       "name": "OpenAI",        "base_url": "https://api.openai.com/v1",  "model": "gpt-4o" },
  { "id": "ollama-local", "name": "Ollama (local)", "base_url": "http://localhost:11434/v1",  "model": "llava:7b" }
]
```

> **`base_url` convention**: Always include the version path (e.g. `/v1`). The system constructs the full endpoint URL as `{base_url}/chat/completions`. If the server's URL already includes a path (e.g. `http://localhost:8142/v1`), use it as-is.

#### Scenario: Default endpoint list on first run
- **WHEN** the application launches for the first time
- **THEN** `ai.endpoints` SHALL contain exactly the two default entries (OpenAI and Ollama local)

#### Scenario: Default endpoint list after reset
- **WHEN** the user resets preferences to defaults
- **THEN** `ai.endpoints` SHALL be restored to the two default entries

#### Scenario: User adds a custom endpoint
- **WHEN** the user adds a new endpoint in Settings with name, base URL, and model
- **THEN** the entry SHALL be appended to `ai.endpoints` with a generated 8-char hex UUID as `id`
- **THEN** the new endpoint SHALL appear in the provider selector immediately

#### Scenario: User removes an endpoint
- **WHEN** the user deletes an endpoint from the list
- **THEN** the entry SHALL be removed from `ai.endpoints`
- **THEN** if the deleted endpoint was the `defaultLlmProvider`, the default SHALL revert to `"claude"`
- **THEN** any stored API key for that endpoint SHALL be deleted from the keychain (account `endpoint-{id}`)

#### Scenario: Default LLM provider selected
- **WHEN** the user invokes LLM vision analysis without explicitly choosing a provider
- **THEN** the application SHALL use the provider specified by `defaultLlmProvider`

---

### Requirement: OS Keychain Storage

All API keys SHALL be stored in the operating system keychain using the `keyring` crate with service name `fotos`. Two namespaces are in use:

- **Provider dispatch string**: `"endpoint:{id}"` (colon) — used in code as the provider identifier
- **Keychain account name**: `"endpoint-{id}"` (hyphen) — used as the OS keychain account (colons are unsafe in some keychain backends)

The following accounts SHALL be used:

| Provider | Service | Account |
|---|---|---|
| Anthropic | `fotos` | `anthropic-api-key` |
| Google | `fotos` | `google-api-key` |
| User-defined endpoint | `fotos` | `endpoint-{id}` |
| ~~OpenAI (v1, deprecated)~~ | ~~`fotos`~~ | ~~`openai-api-key`~~ → migrated to `endpoint-openai` |

#### Scenario: Store API key for user-defined endpoint
- **WHEN** the user provides an API key for a user-defined endpoint
- **THEN** the application SHALL store the key in the OS keychain under service `fotos`, account `endpoint-{id}`

#### Scenario: Retrieve API key for endpoint request
- **WHEN** the system makes a request to a user-defined endpoint that has an API key configured
- **THEN** it SHALL retrieve the key from the keychain using account `endpoint-{id}` and send it as `Authorization: Bearer {key}`

#### Scenario: No API key for local endpoint
- **WHEN** no keychain entry exists for account `endpoint-{id}`
- **THEN** the system SHALL send the request without an `Authorization` header

---

### Requirement: Settings Schema Versioning

The current schema version SHALL be `2`.

**Migration v1 → v2:**

1. Read `ai.openaiModel` (default `"gpt-4o"`); create endpoint `{ "id": "openai", "name": "OpenAI", "base_url": "https://api.openai.com/v1", "model": <value> }` and append to `ai.endpoints`.
2. Read `ai.ollamaUrl` and `ai.ollamaModel`; normalize the URL: if `ollamaUrl` ends with `/v1`, use it as-is; otherwise append `/v1`. Create endpoint `{ "id": "ollama-local", "name": "Ollama (local)", "base_url": <normalized>, "model": <ollamaModel> }` and append to `ai.endpoints`.
3. If `ai.defaultLlmProvider` is `"openai"`, set it to `"endpoint:openai"`.
4. If `ai.defaultLlmProvider` is `"ollama"`, set it to `"endpoint:ollama-local"`.
5. Remove keys `ai.openaiModel`, `ai.ollamaUrl`, `ai.ollamaModel` from the store.
6. Attempt to move keychain entry `openai-api-key` → `endpoint-openai` (read old, write new, delete old). If this step fails, log a warning and continue — the migration is best-effort; the user can re-enter the key in Settings.
7. Set `_schemaVersion` to `2`.

#### Scenario: v1 to v2 migration preserves OpenAI config
- **WHEN** a v1 store has `openaiModel: "gpt-4o-mini"` and `defaultLlmProvider: "openai"`
- **THEN** after migration an endpoint `{ "id": "openai", "name": "OpenAI", "model": "gpt-4o-mini" }` SHALL exist in `ai.endpoints`
- **THEN** `defaultLlmProvider` SHALL be `"endpoint:openai"`

#### Scenario: v1 to v2 migration preserves Ollama config
- **WHEN** a v1 store has `ollamaUrl: "http://localhost:8142"` and `ollamaModel: "library/llava"`
- **THEN** after migration an endpoint `{ "id": "ollama-local", "name": "Ollama (local)", "base_url": "http://localhost:8142/v1", "model": "library/llava" }` SHALL exist in `ai.endpoints`

#### Scenario: v1 to v2 migration handles ollamaUrl already ending in /v1
- **WHEN** a v1 store has `ollamaUrl: "http://localhost:11434/v1"`
- **THEN** after migration the endpoint `base_url` SHALL be `"http://localhost:11434/v1"` (not `"http://localhost:11434/v1/v1"`)

#### Scenario: v1 to v2 keychain migration fails gracefully
- **WHEN** the keychain entry move from `openai-api-key` to `endpoint-openai` fails
- **THEN** the migration SHALL continue and complete (schema version set to 2)
- **THEN** a warning SHALL be logged indicating the user must re-enter their OpenAI key
