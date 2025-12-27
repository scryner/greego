pub mod provider;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value; // Added import
use std::collections::HashMap;
use std::pin::Pin;

use anyhow::{anyhow, Result};
use futures::Stream; // Added import

use log::debug;
// Removed re-exports as we are defining them here now
pub use provider::{anthropic, config, google, lmstudio, openai, openai_compatible};

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

#[cfg(feature = "apple")]
pub use provider::apple;

pub struct LlmServiceManager {
    services: HashMap<String, Box<dyn LlmService>>,
}

impl Default for LlmServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmServiceManager {
    pub fn new() -> Self {
        let mut services: HashMap<String, Box<dyn LlmService>> = HashMap::new();

        #[cfg(feature = "apple")]
        {
            if let Ok(version) = get_mac_os_version() {
                // Check if version is >= 26.0
                if let Some((major, _)) = parse_version(&version) {
                    if major >= 26 {
                        let service = apple::AppleService::new();
                        services.insert("apple".to_string(), Box::new(service));
                    }
                }
            }
        }

        Self { services }
    }

    pub fn add_service(&mut self, name: String, service: Box<dyn LlmService>) {
        self.services.insert(name.clone(), service);
        let all_services: Vec<_> = self.services.keys().collect();
        debug!("added LLM service: {} / {:?}", name, all_services);
    }

    pub fn delete_service(&mut self, name: &str) {
        self.services.remove(name);
        let all_services: Vec<_> = self.services.keys().collect();
        debug!("removed LLM service: {} / {:?}", name, all_services);
    }

    pub fn list_services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    pub async fn get_all_available_models(&self) -> Vec<String> {
        let mut all_models = Vec::new();
        for (service_name, service) in &self.services {
            match service.get_available_models().await {
                Ok(Some(models)) => {
                    for model in models {
                        all_models.push(format!("{}/{}", service_name, model));
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    log::error!("Failed to get models for {}: {}", service_name, e);
                }
            }
        }
        all_models.sort();
        debug!("get_all_available_models: {:?}", all_models);
        all_models
    }

    pub async fn chat_completion(
        &self,
        service_name: &str,
        model_name: &str,
        input: LlmInput,
    ) -> Result<LlmOutput> {
        let service = self
            .services
            .get(service_name)
            .ok_or(anyhow!("Service not found"))?;
        service.chat_completion(model_name, input).await
    }

    pub async fn chat_stream(
        &self,
        service_name: &str,
        model_name: &str,
        input: LlmInput,
    ) -> Result<
        Pin<
            Box<
                dyn futures::Stream<Item = Result<LlmStreamChunk, anyhow::Error>>
                    + std::marker::Send,
            >,
        >,
    > {
        let service = self
            .services
            .get(service_name)
            .ok_or(anyhow!("Service not found"))?;
        service.chat_stream(model_name, input).await
    }
}

#[cfg(feature = "apple")]
fn get_mac_os_version() -> Result<String> {
    use std::process::Command;
    let output = Command::new("sw_vers").arg("-productVersion").output()?;

    if output.status.success() {
        let version = String::from_utf8(output.stdout)?;
        Ok(version.trim().to_string())
    } else {
        Err(anyhow!("Failed to get macOS version"))
    }
}

#[cfg(feature = "apple")]
fn parse_version(version: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        Some((major, minor))
    } else if parts.len() == 1 {
        let major = parts[0].parse().ok()?;
        Some((major, 0))
    } else {
        None
    }
}
