use crate::llm::openai_compatible::OpenAiCompatibleService;
use crate::llm::{LlmInput, LlmOutput, LlmService, LlmStreamChunk};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::time::Duration;

pub struct CustomService {
    inner: OpenAiCompatibleService,
    model: String,
}

impl CustomService {
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        model: String,
        timeout: Duration,
    ) -> Self {
        Self {
            inner: OpenAiCompatibleService::new(base_url, api_key, timeout),
            model,
        }
    }
}

#[async_trait]
impl LlmService for CustomService {
    async fn chat_completion(&self, _model: &str, input: LlmInput) -> anyhow::Result<LlmOutput> {
        self.inner.chat_completion(&self.model, input).await
    }

    async fn chat_stream(
        &self,
        _model: &str,
        input: LlmInput,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<LlmStreamChunk>> + Send>>> {
        self.inner.chat_stream(&self.model, input).await
    }

    async fn get_available_models(&self) -> anyhow::Result<Option<Vec<String>>> {
        Ok(Some(vec![self.model.clone()]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Message, Role};
    use std::time::Duration;

    #[tokio::test]
    #[ignore]
    async fn test_custom_completion() {
        let service = CustomService::new(
            "http://localhost:1234/v1".to_string(), // Example URL: LMStudio example
            Some("sk-test".to_string()),
            "gpt-oss-120b".to_string(),
            Duration::from_secs(60),
        );

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello"),
        };

        // The model arg here is technically ignored by our implementation in favor of the configured one,
        // but we pass it anyway.
        let result = service.chat_completion("ignored", input).await;

        match result {
            Ok(output) => {
                println!("Success! Output: {:?}", output);
            }
            Err(e) => panic!("Failed: {}", e),
        }
    }
}
