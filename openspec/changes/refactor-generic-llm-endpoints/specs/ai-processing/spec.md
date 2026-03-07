## MODIFIED Requirements

### Requirement: LLM Vision Analysis

The system SHALL support sending a screenshot image with an optional text prompt to an LLM vision model for analysis. The system SHALL return the LLM response text, the model name used, and the token count.

The system SHALL support two categories of LLM provider:

**Named providers** — built-in implementations with provider-specific wire formats:
- `claude` — Anthropic API (unique auth headers and request shape)
- `gemini` — Google Gemini API (URL-embedded key, unique request shape)

**User-defined endpoints** — any OpenAI-compatible inference server, identified by the string `"endpoint:{id}"` (colon-separated). Each endpoint is configured with a name, base URL, model, and optional API key stored in the keychain under account `"endpoint-{id}"` (hyphen-separated). The system SHALL send requests to `{base_url}/chat/completions` using the OpenAI chat-completions wire format, with the image encoded as a base64 data URL in the `image_url` content block. If no API key is stored for the endpoint, the `Authorization` header SHALL be omitted.

The `analyze_llm` command SHALL select the provider by the `provider` string: `"claude"`, `"gemini"`, or `"endpoint:{id}"`. For `"endpoint:{id}"`, the system SHALL look up the endpoint configuration from the `ai.endpoints` list in settings.

This design allows users to connect any OpenAI-compatible service (OpenAI, ramalama, LM Studio, Groq, Together AI, Ollama v0.1.24+) without code changes.

#### Scenario: Analyze screenshot with Claude
- **WHEN** the user invokes `analyze_llm` with provider set to `claude`
- **THEN** the system SHALL send the image and prompt to the Anthropic API using the configured Claude model
- **THEN** the system SHALL return an `LlmResponse` containing `response`, `model`, and `tokens_used`

#### Scenario: Analyze screenshot with Gemini
- **WHEN** the user invokes `analyze_llm` with provider set to `gemini`
- **THEN** the system SHALL send the image and prompt to the Google Gemini API using the configured Gemini model
- **THEN** the system SHALL return an `LlmResponse` containing `response`, `model`, and `tokens_used`

#### Scenario: Analyze screenshot with a user-defined endpoint
- **WHEN** the user invokes `analyze_llm` with provider set to `endpoint:{id}`
- **THEN** the system SHALL look up the endpoint by `id` in `ai.endpoints`
- **THEN** the system SHALL POST to `{base_url}/chat/completions` with the OpenAI chat-completions payload
- **THEN** the system SHALL return an `LlmResponse` containing `response`, `model`, and `tokens_used`

#### Scenario: Endpoint without API key (local server)
- **WHEN** a user-defined endpoint has no API key configured in the keychain
- **THEN** the system SHALL send the request without an `Authorization` header
- **THEN** the request SHALL succeed if the server does not require authentication

#### Scenario: Unknown endpoint id
- **WHEN** `analyze_llm` is called with `endpoint:{id}` and no endpoint with that id exists in `ai.endpoints`
- **THEN** the system SHALL return an error: `"Endpoint '{id}' not found"`

#### Scenario: Default prompt when none provided
- **WHEN** the user invokes `analyze_llm` without specifying a prompt
- **THEN** the system SHALL use the default "Describe" prompt: "Describe what you see in this screenshot in detail."

---

## REMOVED Requirements

The Ollama `/api/generate` adapter is removed. Ollama v0.1.24+ exposes `/v1/chat/completions`; users SHALL configure Ollama as a user-defined endpoint with base URL `http://localhost:11434/v1`.

---

## MODIFIED Requirements

### Requirement: Image Size Limits for LLM Vision

Provider limits for named providers remain unchanged. For user-defined endpoints the system SHALL apply a fixed safe default of 2048px longest edge and 20 MB payload.

| Provider | Max Image Dimension | Max Payload Size |
|---|---|---|
| Claude | 8192px (longest edge) | 20 MB |
| Gemini | 3072px (longest edge) | 20 MB |
| User-defined endpoint | 2048px (longest edge) | 20 MB |

#### Scenario: Image exceeds default endpoint dimension limit
- **WHEN** a screenshot is 4000x3000 pixels and a user-defined endpoint is selected
- **THEN** the system SHALL resize the image to 2048px on its longest edge before sending

#### Scenario: Image within default endpoint limit
- **WHEN** a screenshot is 1920x1080 pixels and a user-defined endpoint is selected
- **THEN** the system SHALL send the image at its original dimensions without resizing

---

## ADDED Requirements

### Requirement: Endpoint Connection Test

The system SHALL support testing connectivity and authentication for user-defined endpoints via a `test_api_key` command. For `endpoint:{id}` providers, the test SHALL send `GET {base_url}/models` with the stored API key (or no `Authorization` header if no key is configured). An HTTP 200 response SHALL be treated as success. Any other HTTP status or connection error SHALL be returned as a structured error.

#### Scenario: Test succeeds for reachable endpoint
- **WHEN** the user tests an endpoint and the server responds with HTTP 200 to `GET {base_url}/models`
- **THEN** the system SHALL return a success result

#### Scenario: Test fails for unreachable endpoint
- **WHEN** the user tests an endpoint and the connection is refused or times out
- **THEN** the system SHALL return an error with category `network_error` and a descriptive message

#### Scenario: Test fails for invalid API key
- **WHEN** the user tests an endpoint and the server responds with HTTP 401 or 403
- **THEN** the system SHALL return an error indicating the API key is invalid or missing
