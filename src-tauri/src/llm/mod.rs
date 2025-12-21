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

pub struct LlmModel {
    pub model: String,
    pub service: Box<dyn LlmService>,
}

pub struct LlmServiceManager {
    services: HashMap<String, LlmModel>,
}

impl LlmServiceManager {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn add_model(&mut self, name: String, service: LlmModel) {
        self.services.insert(name, service);
    }

    pub async fn chat_completion(&self, model_name: &str, input: LlmInput) -> Result<LlmOutput> {
        let model = self
            .services
            .get(model_name)
            .ok_or(anyhow!("Service not found"))?;
        model.service.chat_completion(&model.model, input).await
    }

    pub async fn chat_stream(
        &self,
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
        let model = self
            .services
            .get(model_name)
            .ok_or(anyhow!("Service not found"))?;
        model.service.chat_stream(&model.model, input).await
    }
}
