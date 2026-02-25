/// Local LLM via Ollama.
///
/// Connects to a local Ollama instance for image analysis
/// without requiring cloud API keys.
use anyhow::{bail, Result};
use std::time::{Duration, Instant};

const TIMEOUT_SECS: u64 = 30;

pub struct OllamaConfig {
    pub url: String,
    pub model: String,
}

/// Analyze an image with a local Ollama instance.
///
/// `image_b64` must be a base64-encoded JPEG (from `compress::compress_for_llm`).
pub async fn analyze(
    image_b64: &str,
    prompt: &str,
    config: &OllamaConfig,
) -> Result<super::llm::LlmOutput> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()?;

    let url = format!("{}/api/generate", config.url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": config.model,
        "prompt": prompt,
        "images": [image_b64],
        "stream": false
    });

    let start = Instant::now();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Ollama request failed (is Ollama running at {}?): {e}", config.url))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await?;

    if !status.is_success() {
        let msg = json["error"].as_str().unwrap_or("unknown error");
        bail!("Ollama error {status}: {msg}");
    }

    let response = json["response"].as_str().unwrap_or("").to_string();

    Ok(super::llm::LlmOutput {
        response,
        model: config.model.clone(),
        tokens_used: 0, // Ollama does not report token counts in /api/generate
        latency_ms: start.elapsed().as_millis() as u64,
    })
}
