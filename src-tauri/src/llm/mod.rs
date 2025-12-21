pub mod provider;

use std::collections::HashMap;
use std::pin::Pin;

use anyhow::{anyhow, Result};

pub use provider::{
    anthropic, config, google, lmstudio, openai, openai_compatible, ContentPart, LlmInput,
    LlmOutput, LlmService, LlmStreamChunk, Message, Role, TokenUsage,
};

#[cfg(feature = "apple")]
pub use provider::apple;

pub struct LlmServiceManager {
    services: HashMap<String, Box<dyn LlmService>>,
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
        self.services.insert(name, service);
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
        service.chat_completion(&model_name, input).await
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
        service.chat_stream(&model_name, input).await
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
