use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "service", rename_all = "lowercase")]
pub enum LlmServiceConfig {
    Google(GoogleConfig),
    OpenAI(OpenAIConfig),
    Anthropic(AnthropicConfig),
    Ollama(OllamaConfig),
    LMStudio(LMStudioConfig),
    Custom(CustomConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LMStudioConfig {
    pub base_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
}
