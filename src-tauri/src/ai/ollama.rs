/// Local LLM via Ollama.
///
/// Connects to a local Ollama instance for image analysis
/// without requiring cloud API keys.
use anyhow::Result;

pub struct OllamaConfig {
    pub url: String,
    pub model: String,
}

pub async fn analyze(
    _image: &image::DynamicImage,
    _prompt: &str,
    _config: &OllamaConfig,
) -> Result<super::llm::LlmOutput> {
    // TODO: send image to Ollama REST API at configured URL
    anyhow::bail!("Ollama analysis not yet implemented")
}
