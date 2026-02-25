/// Cloud LLM vision analysis.
///
/// Supports Claude (Anthropic), GPT-4o (OpenAI), and Gemini (Google)
/// for image understanding and analysis.
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

const TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    Claude { model: String },
    OpenAI { model: String },
    Gemini { model: String },
}

pub struct LlmOutput {
    pub response: String,
    pub model: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
}

/// Analyze an image with a cloud LLM provider.
///
/// `image_b64` must be a base64-encoded JPEG (from `compress::compress_for_llm`).
pub async fn analyze(
    image_b64: &str,
    prompt: &str,
    provider: &LlmProvider,
    api_key: &str,
) -> Result<LlmOutput> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()?;

    match provider {
        LlmProvider::Claude { model } => {
            analyze_claude(&client, image_b64, prompt, model, api_key).await
        }
        LlmProvider::OpenAI { model } => {
            analyze_openai(&client, image_b64, prompt, model, api_key).await
        }
        LlmProvider::Gemini { model } => {
            analyze_gemini(&client, image_b64, prompt, model, api_key).await
        }
    }
}

async fn analyze_claude(
    client: &reqwest::Client,
    image_b64: &str,
    prompt: &str,
    model: &str,
    api_key: &str,
) -> Result<LlmOutput> {
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/jpeg",
                        "data": image_b64
                    }
                },
                {
                    "type": "text",
                    "text": prompt
                }
            ]
        }]
    });

    let start = Instant::now();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await?;

    if !status.is_success() {
        let msg = json["error"]["message"].as_str().unwrap_or("unknown error");
        bail!("Anthropic API error {status}: {msg}");
    }

    let response = json["content"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c["text"].as_str())
        .unwrap_or("")
        .to_string();

    let tokens_used = (json["usage"]["input_tokens"].as_u64().unwrap_or(0)
        + json["usage"]["output_tokens"].as_u64().unwrap_or(0)) as u32;

    Ok(LlmOutput {
        response,
        model: model.to_string(),
        tokens_used,
        latency_ms: start.elapsed().as_millis() as u64,
    })
}

async fn analyze_openai(
    client: &reqwest::Client,
    image_b64: &str,
    prompt: &str,
    model: &str,
    api_key: &str,
) -> Result<LlmOutput> {
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

    let start = Instant::now();
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await?;

    if !status.is_success() {
        let msg = json["error"]["message"].as_str().unwrap_or("unknown error");
        bail!("OpenAI API error {status}: {msg}");
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

async fn analyze_gemini(
    client: &reqwest::Client,
    image_b64: &str,
    prompt: &str,
    model: &str,
    api_key: &str,
) -> Result<LlmOutput> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    );

    let body = serde_json::json!({
        "contents": [{
            "parts": [
                {
                    "inlineData": {
                        "mimeType": "image/jpeg",
                        "data": image_b64
                    }
                },
                {
                    "text": prompt
                }
            ]
        }]
    });

    let start = Instant::now();
    let resp = client.post(&url).json(&body).send().await?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await?;

    if !status.is_success() {
        let msg = json["error"]["message"].as_str().unwrap_or("unknown error");
        bail!("Gemini API error {status}: {msg}");
    }

    let response = json["candidates"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c["content"]["parts"].as_array())
        .and_then(|p| p.first())
        .and_then(|p| p["text"].as_str())
        .unwrap_or("")
        .to_string();

    let tokens_used = json["usageMetadata"]["totalTokenCount"]
        .as_u64()
        .unwrap_or(0) as u32;

    Ok(LlmOutput {
        response,
        model: model.to_string(),
        tokens_used,
        latency_ms: start.elapsed().as_millis() as u64,
    })
}
