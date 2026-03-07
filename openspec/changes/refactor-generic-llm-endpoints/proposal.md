# Change Proposal: refactor-generic-llm-endpoints

## Summary

Replace the four hardcoded LLM provider implementations (OpenAI, Ollama, Claude, Gemini) with a two-tier model:

- **Named providers** — Claude (Anthropic) and Gemini (Google) remain as built-in providers because their wire formats are unique (custom auth headers, non-OpenAI request/response shapes).
- **User-defined endpoints** — A configurable list of OpenAI-compatible endpoints replaces the hardcoded OpenAI and Ollama providers. Each endpoint carries a name, base URL, model, and optional API key. This covers OpenAI, local llama-server (ramalama), LM Studio, Groq, Together AI, Ollama v2+, and any future OpenAI-compatible service — without code changes.

## Motivation

The current design requires a code change to add any new LLM provider. Users running local inference servers (e.g. ramalama on port 8142 serving llava via llama.cpp) have no way to connect them to Fotos today. The OpenAI-compatible API (`/v1/chat/completions`) is now a de-facto standard supported by virtually all local and third-party inference servers. A single generic adapter covers this entire class of providers.

## Scope

| Area | Change |
|---|---|
| `src-tauri/src/ai/llm.rs` | Drop `LlmProvider::OpenAI`; add `openai_compat::analyze(base_url, model, api_key)` |
| `src-tauri/src/ai/ollama.rs` | Remove (Ollama v2 exposes `/v1/chat/completions`) |
| `src-tauri/src/commands/ai.rs` | Update dispatch: `claude` / `gemini` / `endpoint:{id}` |
| `src-tauri/src/commands/settings.rs` | `AiSettings` gets `endpoints: Vec<LlmEndpoint>` replacing fixed openai/ollama fields |
| `src-ui/js/ui/settings.js` | Dynamic endpoint list (add/remove/edit rows) replaces fixed openai/ollama fields |
| `settings-credentials` spec | AI preferences schema updated; schema version bumped to 2 |
| `ai-processing` spec | LLM provider requirement updated to reflect two-tier model |

## Not In Scope

- Claude API format changes
- Gemini API format changes
- OCR, PII, capture, annotation — unaffected
- MCP server — unaffected

## Design Decisions

### Why keep Claude and Gemini as named providers?

Their wire formats are genuinely different from OpenAI:
- Claude uses `x-api-key` + `anthropic-version` headers and a `content[].source` image format
- Gemini uses a URL-embedded API key and `inlineData.mimeType` image format

Forcing them through the generic adapter would require mapping logic more complex than just keeping dedicated functions.

### Why drop the Ollama `/api/generate` adapter?

Ollama v2+ exposes `/v1/chat/completions` at the same base URL. The old `/api/generate` endpoint is legacy. Users pointing at Ollama can use a custom endpoint with base URL `http://localhost:11434` and their model name.

### Endpoint ID and keychain key

Each custom endpoint has a user-assigned `id` — a v4 UUID truncated to 8 hex characters, generated on creation. Two namespaces are used:

- **Provider dispatch string**: `"endpoint:{id}"` (colon-separated) — passed to `analyze_llm` and stored as `defaultLlmProvider`
- **Keychain account name**: `"endpoint-{id}"` (hyphen-separated) — used as the account key in the OS keychain (colons are unsafe in some keychain backends)

If no API key is needed (local server), no keychain entry is created. The system checks for a keychain entry at request time; absence means no `Authorization` header is sent.

### Schema migration (v1 → v2)

On upgrade, the migration function reads the old `ollamaUrl` + `ollamaModel` fields and creates a pre-populated endpoint entry named "Ollama (local)" with those values. The old `openaiModel` field creates an "OpenAI" endpoint entry with base URL `https://api.openai.com/v1`. The `defaultLlmProvider` field is migrated: `openai` → `endpoint:{openai-entry-id}`, `ollama` → `endpoint:{ollama-entry-id}`, `claude`/`gemini` remain unchanged.

## Risks

- **Breaking change for existing provider strings**: Any code passing `"openai"` or `"ollama"` as the provider string (e.g. MCP clients, test scripts) will break after the refactor. The migration handles the settings store, but external callers must update their provider strings.
- **Migration atomicity**: Migration is best-effort. If the keychain entry move (step 6) fails, the schema is still bumped to v2 and the OpenAI key is orphaned. Recovery: user re-enters the key in Settings.
- **Ollama minimum version**: The OpenAI-compatible endpoint (`/v1/chat/completions`) requires Ollama v0.1.24+. Users on older versions will get connection errors.
- **Ollama URL migration edge case**: If an existing `ollamaUrl` already ends with `/v1`, the migration must not append another `/v1`. A trailing `/v1` check is required.
- **Text-only model misconfiguration**: A user may configure a text-only model for image analysis. The error from the server should surface clearly rather than returning an empty response.
