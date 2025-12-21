pub mod anthropic;
pub mod config;
pub mod google;
pub mod lmstudio;
pub mod openai;
pub mod openai_compatible;

#[cfg(feature = "apple")]
pub mod apple;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "snake_case")]
pub enum ContentPart {
    Text(String),
    ImageUrl(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentPart>,
}

impl Message {
    pub fn new_text(role: Role, text: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![ContentPart::Text(text.into())],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInput {
    pub system_prompt: Option<String>,
    pub history: Vec<Message>,
    pub user_input: Message,
    // Future optional definitions:
    // pub temperature: Option<f32>,
    // pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmOutput {
    pub content: Vec<ContentPart>,
    pub usage: Option<TokenUsage>,
    pub raw: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStreamChunk {
    pub content: String, // Simplified for now, just text delta
    pub usage: Option<TokenUsage>,
}

use futures::Stream;
use std::pin::Pin;

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn chat_completion(&self, model: &str, input: LlmInput) -> anyhow::Result<LlmOutput>;

    async fn chat_stream(
        &self,
        model: &str,
        input: LlmInput,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<LlmStreamChunk>> + Send>>>;

    async fn get_available_models(&self) -> anyhow::Result<Option<Vec<String>>> {
        Ok(None)
    }
}
