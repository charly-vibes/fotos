# Capability: AI Processing Pipeline

## Purpose

AI-powered processing features for Fotos (io.github.charly.fotos), including OCR text extraction, PII detection and auto-blur, and LLM vision analysis with multiple provider support.

## Requirements

### Requirement: OCR Text Extraction via Tesseract

The system SHALL extract text from screenshot images using Tesseract OCR. The OCR engine SHALL return both the full extracted text and per-word bounding boxes with confidence scores. The system SHALL convert the input image to grayscale before passing it to Tesseract. The system SHALL use PSM mode 3 (fully automatic page segmentation) by default.

#### Scenario: Extract text with default language
- **WHEN** the user invokes OCR on a screenshot containing English text
- **THEN** the system SHALL run Tesseract with language `eng` and PSM mode 3
- **THEN** the system SHALL return an `OcrResult` containing the full extracted text and an array of regions, each with `text`, `x`, `y`, `w`, `h`, and `confidence` fields

#### Scenario: Extract text with configurable language
- **WHEN** the user invokes OCR and specifies a language code (e.g., `deu`, `jpn`)
- **THEN** the system SHALL use the specified language for Tesseract processing
- **THEN** if the requested language data is not available locally, the system SHALL download it on demand to the app data directory

#### Scenario: Default language when none specified
- **WHEN** the user invokes OCR without specifying a language
- **THEN** the system SHALL default to `eng`

#### Scenario: Bundled tessdata
- **WHEN** the application is installed
- **THEN** the `eng.traineddata` file SHALL be bundled in the application resources under `resources/tessdata/`

---

### Requirement: PII Auto-Detection

The system SHALL detect personally identifiable information (PII) in screenshot text by running OCR first, then applying regex pattern matching against the extracted text regions. The system SHALL return a list of matches, each containing the bounding box coordinates and the PII type.

The system SHALL include built-in regex patterns for the following PII types:

| PII Type | Pattern |
|---|---|
| Email | `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}` |
| Phone (US) | `\b(\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b` |
| SSN | `\b\d{3}-\d{2}-\d{4}\b` |
| Credit Card | `\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b` |
| API Key | `\b(sk-[a-zA-Z0-9]{32,}\|AKIA[A-Z0-9]{16}\|ghp_[a-zA-Z0-9]{36})\b` |
| IP Address | `\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b` |
| AWS ARN | `\barn:aws:[a-z0-9-]+:[a-z0-9-]*:\d{12}:` |

> **Regex notation**: The `\|` characters in the API Key pattern represent regex alternation (`|`). The backslash is a Markdown table rendering artifact. Implementations MUST use unescaped `|` for alternation.

> **Limitations**: These patterns are best-effort heuristics and will NOT catch all PII (e.g., names, addresses, non-US phone formats, obfuscated credentials). The UI SHOULD inform the user that auto-detection is not exhaustive and manual review is recommended before sharing screenshots.

#### Scenario: Detect email addresses in screenshot
- **WHEN** OCR extracts text containing an email address (e.g., `user@example.com`)
- **THEN** the system SHALL return a match with `pii_type` set to `email` and the bounding box (`x`, `y`, `w`, `h`) of the matching text region

#### Scenario: Detect multiple PII types
- **WHEN** OCR extracts text containing a phone number, an SSN, and a credit card number
- **THEN** the system SHALL return separate matches for each, with `pii_type` set to `phone`, `ssn`, and `credit_card` respectively, each with its own bounding box

#### Scenario: Detect API keys
- **WHEN** OCR extracts text containing a string matching an API key pattern (e.g., `sk-...`, `AKIA...`, `ghp_...`)
- **THEN** the system SHALL return a match with `pii_type` set to `api_key` and the corresponding bounding box

#### Scenario: Detect IP addresses
- **WHEN** OCR extracts text containing an IP address (e.g., `192.168.1.1`)
- **THEN** the system SHALL return a match with `pii_type` set to `ip_address` and the corresponding bounding box

#### Scenario: Detect AWS ARNs
- **WHEN** OCR extracts text containing an AWS ARN (e.g., `arn:aws:iam:us-east-1:123456789012:...`)
- **THEN** the system SHALL return a match with `pii_type` set to `aws_arn` and the corresponding bounding box

#### Scenario: No PII found
- **WHEN** OCR extracts text that does not match any PII patterns
- **THEN** the system SHALL return an empty list of matches

---

### Requirement: PII Auto-Blur

The system SHALL automatically create blur annotations at the coordinates of detected PII regions. The `auto_blur_pii` command SHALL run the full PII detection pipeline (OCR followed by pattern matching) and then generate blur annotations for each detected PII region.

#### Scenario: Auto-blur detected PII
- **WHEN** the user invokes `auto_blur_pii` on a screenshot
- **THEN** the system SHALL run OCR and PII detection on the image
- **THEN** for each detected PII region, the system SHALL create a blur annotation at the region's bounding box coordinates
- **THEN** the system SHALL return the list of blur regions, each containing `x`, `y`, `w`, `h`, and `pii_type`

#### Scenario: No PII to blur
- **WHEN** the user invokes `auto_blur_pii` on a screenshot containing no PII
- **THEN** the system SHALL return an empty list of blur regions

#### Scenario: Frontend applies blur annotations
- **WHEN** the backend returns blur regions from `auto_blur_pii`
- **THEN** the frontend SHALL auto-create blur annotation objects at the returned coordinates on the annotation canvas

---

### Requirement: LLM Vision Analysis

The system SHALL support sending a screenshot image with an optional text prompt to an LLM vision model for analysis. The system SHALL return the LLM response text, the model name used, and the token count.

All LLM providers MUST implement a common `LlmProvider` async trait that defines:

- `analyze(image: &[u8], prompt: &str, model: &str) -> Result<LlmResponse>` -- send image and prompt, return analysis
- `name() -> &str` -- provider identifier string (e.g., `"claude"`, `"openai"`)

The system SHALL ship with four built-in provider implementations: Claude (Anthropic), OpenAI, Gemini (Google), and Ollama (local). The `analyze_llm` command SHALL select the provider by name from a registry of available implementations. This trait-based design allows adding new providers without modifying the dispatch logic.

#### Scenario: Analyze screenshot with Claude
- **WHEN** the user invokes `analyze_llm` with provider set to `claude`
- **THEN** the system SHALL send the image and prompt to the Anthropic API using the configured Claude model
- **THEN** the system SHALL return an `LlmResponse` containing `response`, `model`, and `tokens_used`

#### Scenario: Analyze screenshot with OpenAI
- **WHEN** the user invokes `analyze_llm` with provider set to `openai`
- **THEN** the system SHALL send the image and prompt to the OpenAI API using the configured OpenAI model
- **THEN** the system SHALL return an `LlmResponse` containing `response`, `model`, and `tokens_used`

#### Scenario: Analyze screenshot with Gemini
- **WHEN** the user invokes `analyze_llm` with provider set to `gemini`
- **THEN** the system SHALL send the image and prompt to the Google Gemini API using the configured Gemini model
- **THEN** the system SHALL return an `LlmResponse` containing `response`, `model`, and `tokens_used`

#### Scenario: Analyze screenshot with Ollama
- **WHEN** the user invokes `analyze_llm` with provider set to `ollama`
- **THEN** the system SHALL send the image and prompt to the Ollama API at the configured URL using the configured Ollama model
- **THEN** the system SHALL return an `LlmResponse` containing `response`, `model`, and `tokens_used`

#### Scenario: Default prompt when none provided
- **WHEN** the user invokes `analyze_llm` without specifying a prompt
- **THEN** the system SHALL use the default "Describe" prompt: "Describe what you see in this screenshot in detail."

---

### Requirement: Default Prompt Templates

The system SHALL provide a set of built-in prompt templates for LLM vision analysis. Each template SHALL have a name and a corresponding prompt string.

| Template Name | Prompt |
|---|---|
| Describe | "Describe what you see in this screenshot in detail." |
| Extract Code | "Extract all code visible in this screenshot. Return only the code, properly formatted." |
| Bug Report | "Analyze this screenshot and generate a bug report with: summary, steps to reproduce (inferred), expected vs actual behavior, and severity assessment." |
| Accessibility | "Generate alt-text for this screenshot suitable for screen readers." |
| Custom | User-provided prompt text |

#### Scenario: Use Describe prompt
- **WHEN** the user selects the "Describe" prompt template
- **THEN** the system SHALL send the prompt "Describe what you see in this screenshot in detail." to the LLM provider

#### Scenario: Use Extract Code prompt
- **WHEN** the user selects the "Extract Code" prompt template
- **THEN** the system SHALL send the prompt "Extract all code visible in this screenshot. Return only the code, properly formatted." to the LLM provider

#### Scenario: Use Bug Report prompt
- **WHEN** the user selects the "Bug Report" prompt template
- **THEN** the system SHALL send the prompt "Analyze this screenshot and generate a bug report with: summary, steps to reproduce (inferred), expected vs actual behavior, and severity assessment." to the LLM provider

#### Scenario: Use Accessibility prompt
- **WHEN** the user selects the "Accessibility" prompt template
- **THEN** the system SHALL send the prompt "Generate alt-text for this screenshot suitable for screen readers." to the LLM provider

#### Scenario: Use Custom prompt
- **WHEN** the user provides a custom prompt string
- **THEN** the system SHALL send the user-provided prompt text verbatim to the LLM provider

---

### Requirement: LLM Provider Configuration

The system SHALL read all LLM provider configuration (model names, Ollama URL, API keys) from the settings and credentials defined in the **settings-credentials** spec. This spec does not re-define those values; the settings-credentials spec is the single source of truth for default model names, Ollama endpoint configuration, and API key storage.

The AI processing pipeline SHALL use the configured values as follows:
- The active model name for each provider is read from the `ai` preferences section
- API keys are retrieved from the OS keychain (service `fotos`)
- Ollama URL and model are read from the `ai` preferences section

#### Scenario: Provider configuration applied
- **WHEN** the user invokes LLM vision analysis
- **THEN** the system SHALL read the provider's model name and (if applicable) API key from the settings-credentials store
- **THEN** the system SHALL use those values for the API request

#### Scenario: Settings change takes effect immediately
- **WHEN** the user changes an LLM provider setting and then invokes analysis
- **THEN** the new setting value SHALL be used for the request without requiring an app restart

---

### Requirement: LLM API Error Handling

The system SHALL handle all LLM API errors gracefully and return structured error information to the caller. Error categories MUST include: authentication failure (invalid or expired API key), rate limiting (429 status), network error (timeout, DNS, connection refused), provider error (500-level responses), and invalid response (malformed or empty body). The system SHALL NOT crash or hang on any API error.

#### Scenario: Authentication failure
- **WHEN** the system sends a request to a cloud LLM provider and receives a 401 or 403 response
- **THEN** the system SHALL return an error with category `auth_error` and a message indicating the API key is invalid or expired

#### Scenario: Rate limit exceeded
- **WHEN** the system sends a request to a cloud LLM provider and receives a 429 response
- **THEN** the system SHALL return an error with category `rate_limited` and include the `Retry-After` header value if present

#### Scenario: Network error
- **WHEN** the system cannot connect to the LLM provider (DNS failure, connection refused, TLS error)
- **THEN** the system SHALL return an error with category `network_error` and a descriptive message within the timeout period

#### Scenario: Provider returns server error
- **WHEN** the LLM provider returns a 500-level response
- **THEN** the system SHALL return an error with category `provider_error` and include the HTTP status code

---

### Requirement: LLM Request Timeout and Cancellation

The system SHALL enforce a configurable timeout on all LLM API requests. The default timeout SHALL be 60 seconds. If the timeout elapses before a response is received, the system SHALL abort the request and return a timeout error. The system SHALL support cancellation of in-progress LLM requests via a `cancel_llm_request` command.

#### Scenario: Request times out
- **WHEN** an LLM API request does not receive a response within the configured timeout period
- **THEN** the system SHALL abort the HTTP request and return an error with category `timeout`

#### Scenario: User cancels in-progress request
- **WHEN** the user invokes `cancel_llm_request` while an LLM request is in flight
- **THEN** the system SHALL abort the HTTP request and return a cancellation acknowledgment

#### Scenario: Default timeout value
- **WHEN** no custom timeout is configured
- **THEN** the system SHALL use a 60-second timeout for LLM API requests

---

### Requirement: Image Size Limits for LLM Vision

The system SHALL validate image dimensions and file size before sending to LLM vision APIs. If the image exceeds a provider's limits, the system SHALL automatically resize the image to fit within the limits while preserving the aspect ratio. The system SHALL NOT send images larger than the provider's maximum payload.

Provider limits:

| Provider | Max Image Dimension | Max Payload Size |
|----------|-------------------|-----------------|
| Claude | 8192px (longest edge) | 20 MB |
| OpenAI | 2048px (longest edge) | 20 MB |
| Gemini | 3072px (longest edge) | 20 MB |
| Ollama | Model-dependent | No hard limit |

#### Scenario: Image exceeds provider dimension limit
- **WHEN** a screenshot is 10000x5000 pixels and the provider is Claude (max 8192px)
- **THEN** the system SHALL resize the image to 8192x4096 pixels (preserving aspect ratio) before sending

#### Scenario: Image within provider limits
- **WHEN** a screenshot is 1920x1080 pixels and the provider is any supported provider
- **THEN** the system SHALL send the image at its original dimensions without resizing

#### Scenario: Ollama uses passthrough
- **WHEN** the provider is Ollama
- **THEN** the system SHALL send the image without resizing, as Ollama models handle their own image scaling
