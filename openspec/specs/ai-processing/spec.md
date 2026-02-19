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

The system SHALL support sending a screenshot image with an optional text prompt to an LLM vision model for analysis. The system SHALL support the following providers: Claude (Anthropic), OpenAI, Gemini (Google), and Ollama (local). The system SHALL return the LLM response text, the model name used, and the token count.

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

The system SHALL allow users to configure the model used for each LLM provider. For the Ollama provider, the system SHALL additionally allow configuration of the server URL and model name. All API keys MUST be stored in the OS keychain and SHALL NOT be stored in configuration files or localStorage.

Default provider configurations:

| Provider | Default Model | Additional Config |
|---|---|---|
| Claude | `claude-sonnet-4-20250514` | API key via OS keychain |
| OpenAI | `gpt-4o` | API key via OS keychain |
| Gemini | `gemini-2.0-flash` | API key via OS keychain |
| Ollama | `llava:7b` | URL: `http://localhost:11434` |

#### Scenario: Configure Claude model
- **WHEN** the user sets the Claude model in settings
- **THEN** the system SHALL use the specified model for all subsequent Claude API calls

#### Scenario: Configure OpenAI model
- **WHEN** the user sets the OpenAI model in settings
- **THEN** the system SHALL use the specified model for all subsequent OpenAI API calls

#### Scenario: Configure Gemini model
- **WHEN** the user sets the Gemini model in settings
- **THEN** the system SHALL use the specified model for all subsequent Gemini API calls

#### Scenario: Configure Ollama URL and model
- **WHEN** the user sets the Ollama URL and model in settings
- **THEN** the system SHALL use the specified URL and model for all subsequent Ollama API calls

#### Scenario: Default Ollama configuration
- **WHEN** the user has not configured Ollama settings
- **THEN** the system SHALL default to URL `http://localhost:11434` and model `llava:7b`

#### Scenario: API key storage
- **WHEN** the user provides an API key for Claude, OpenAI, or Gemini
- **THEN** the system SHALL store the key in the OS keychain under service `fotos`
- **THEN** the system SHALL NOT write the key to any configuration file or localStorage
