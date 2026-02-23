/// Cloud LLM vision analysis.
///
/// Supports Claude (Anthropic), GPT-4o (OpenAI), and Gemini (Google)
/// for image understanding and analysis.
use anyhow::Result;
use serde::{Deserialize, Serialize};

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
}

pub async fn analyze(
    _image: &image::DynamicImage,
    _prompt: &str,
    _provider: &LlmProvider,
    _api_key: &str,
) -> Result<LlmOutput> {
    // TODO: encode image as base64, send to provider API via reqwest
    anyhow::bail!("LLM analysis not yet implemented")
}
