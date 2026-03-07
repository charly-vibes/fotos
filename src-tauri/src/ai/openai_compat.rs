/// Generic OpenAI-compatible vision analyzer.
///
/// Supports any server implementing `/v1/chat/completions` — OpenAI, llama-server,
/// LM Studio, Groq, Together AI, Ollama v2+, and any other compatible service.
use anyhow::{bail, Result};
use std::time::{Duration, Instant};

use super::llm::LlmOutput;

const TIMEOUT_SECS: u64 = 30;

/// Analyze an image using an OpenAI-compatible `/chat/completions` endpoint.
///
/// `base_url` should include the path prefix (e.g. `https://api.openai.com/v1`
/// or `http://localhost:11434/v1`). The function appends `/chat/completions`.
///
/// `api_key` may be empty for local servers that require no authentication.
pub async fn analyze(
    image_b64: &str,
    prompt: &str,
    base_url: &str,
    model: &str,
    api_key: &str,
) -> Result<LlmOutput> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()?;

    let base = base_url.trim_end_matches('/');
    let url = format!("{base}/chat/completions");

    let data_url = format!("data:image/jpeg;base64,{image_b64}");
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image_url",
                    "image_url": { "url": data_url }
                },
                {
                    "type": "text",
                    "text": prompt
                }
            ]
        }]
    });

    let mut req = client.post(&url).json(&body);
    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }

    let start = Instant::now();
    let resp = req.send().await.map_err(|e| {
        anyhow::anyhow!("Request to {url} failed: {e}")
    })?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await?;

    if !status.is_success() {
        let msg = json["error"]["message"].as_str().unwrap_or("unknown error");
        bail!("API error {status}: {msg}");
    }

    let response = json["choices"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c["message"]["content"].as_str())
        .unwrap_or("")
        .to_string();

    let tokens_used = json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;

    Ok(LlmOutput {
        response,
        model: model.to_string(),
        tokens_used,
        latency_ms: start.elapsed().as_millis() as u64,
    })
}
