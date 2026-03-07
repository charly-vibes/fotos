# Tasks: refactor-generic-llm-endpoints

## Backend

- [x] 1.1 Add `src-tauri/src/ai/openai_compat.rs` — generic OpenAI-compatible analyzer (`base_url`, `model`, `api_key`, supports empty key for local servers)
- [x] 1.2 Remove `src-tauri/src/ai/ollama.rs` and its `mod` entry in `ai/mod.rs`
- [x] 1.3 Update `LlmProvider` enum in `llm.rs` — remove `OpenAI` variant; keep `Claude` and `Gemini`
- [x] 1.4 Update `AiSettings` in `settings.rs` — remove `openai_model`, `ollama_url`, `ollama_model`; add `endpoints: Vec<LlmEndpoint>` and `default_llm_provider: String`
- [x] 1.5 Define `LlmEndpoint` struct: `{ id: String, name: String, base_url: String, model: String }`
- [x] 1.6 Update `analyze_llm` command dispatch — handle `"claude"`, `"gemini"`, and `"endpoint:{id}"` patterns; look up endpoint by id from `ai_settings.endpoints`
- [x] 1.7 Update keychain commands (`get_api_key`, `set_api_key`, `delete_api_key`) to accept `endpoint:{id}` as provider key, stored as account `endpoint-{id}` in service `fotos`
- [x] 1.7a Implement `test_api_key` for `endpoint:{id}` providers: send `GET {base_url}/models` with the stored API key (or no auth if none); treat HTTP 200 as success, any other status or connection error as failure
- [x] 1.8 Add schema migration v1→v2 in settings: migrate `openaiModel` → endpoint "OpenAI" (`https://api.openai.com/v1`), migrate `ollamaUrl`+`ollamaModel` → endpoint "Ollama (local)", update `defaultLlmProvider`
- [x] 1.9 Ship a default endpoint list for first-run: "OpenAI" (`https://api.openai.com/v1`, `gpt-4o`) and "Ollama (local)" (`http://localhost:11434`, `llava:7b`)

## Frontend

- [x] 2.1 Update `DEFAULTS.ai` in `settings.js` — replace fixed openai/ollama fields with the two default endpoint entries (OpenAI at `https://api.openai.com/v1` and Ollama local at `http://localhost:11434/v1`) and `defaultLlmProvider: 'claude'`
- [x] 2.2 Update `applyToForm` / `readFromForm` to handle the dynamic endpoint list
- [x] 2.3 Add endpoint list UI in the Settings AI tab: table of rows (name, base URL, model, API key button, delete button) with an "Add endpoint" button
- [x] 2.4 Add endpoint row template: fields for name, base URL, model; "Set Key" / "Delete Key" actions wired to `setApiKey`/`deleteApiKey` with `endpoint:{id}` provider
- [x] 2.5 Update the provider selector in the AI panel to populate from `ai.endpoints` plus `claude` and `gemini` as fixed entries
- [x] 2.6 Update `tauri-bridge.js` if any command signatures changed
- [x] 2.7 Add HTML template for endpoint rows in the Settings AI tab (name input, base URL input, model input, Set Key / Delete Key buttons)

## Spec & Docs

- [x] 3.1 Update `ai-processing` spec delta (LLM provider requirement)
- [x] 3.2 Update `settings-credentials` spec delta (AI preferences schema, keychain accounts, schema version)
- [x] 3.3 Update `CLAUDE.md` / `README.md` if Ollama setup instructions reference `/api/generate`
- [x] 3.4 Note in MCP server docs that `analyze_llm` provider strings `"openai"` and `"ollama"` are replaced by `"endpoint:openai"` and `"endpoint:ollama-local"` after this change (breaking change for MCP callers)
