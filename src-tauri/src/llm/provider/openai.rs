use crate::llm::openai_compatible::OpenAiCompatibleService;
use crate::llm::{LlmInput, LlmOutput, LlmService, LlmStreamChunk};
use async_trait::async_trait;
use futures::Stream;
use serde::Deserialize;
use std::pin::Pin;
use std::time::Duration;

pub struct OpenAiService {
    inner: OpenAiCompatibleService,
}

impl OpenAiService {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        Self {
            inner: OpenAiCompatibleService::new(
                "https://api.openai.com/v1".to_string(),
                Some(api_key),
                timeout,
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ListModelsResponse {
    data: Vec<ModelData>,
}

#[derive(Debug, Deserialize)]
struct ModelData {
    id: String,
}

#[async_trait]
impl LlmService for OpenAiService {
    async fn chat_completion(&self, model: &str, input: LlmInput) -> anyhow::Result<LlmOutput> {
        self.inner.chat_completion(model, input).await
    }

    async fn chat_stream(
        &self,
        model: &str,
        input: LlmInput,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<LlmStreamChunk>> + Send>>> {
        self.inner.chat_stream(model, input).await
    }

    async fn get_available_models(&self) -> anyhow::Result<Option<Vec<String>>> {
        let url = format!("{}/models", self.inner.base_url);

        let mut request_builder = self.inner.client.get(&url);

        if let Some(ref key) = self.inner.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI API Error: {} - {}", status, text));
        }

        let response_body: ListModelsResponse = response.json().await?;
        let models = response_body.data.into_iter().map(|m| m.id).collect();

        Ok(Some(models))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Message, Role};
    use std::env;

    fn get_api_key() -> String {
        env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set")
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_completion() {
        let api_key = get_api_key();
        let service = OpenAiService::new(api_key, Duration::from_secs(60));

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, who are you?"),
        };

        let result = service.chat_completion("gpt-4o", input).await;

        match result {
            Ok(output) => {
                println!("Success! Output: {:?}", output);
                assert!(!output.content.is_empty(), "Content should not be empty");
            }
            Err(e) => panic!("Failed to get completion: {}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_stream() {
        use futures::StreamExt;

        let api_key = get_api_key();
        let service = OpenAiService::new(api_key, Duration::from_secs(60));

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, who are you?"),
        };

        let result = service.chat_stream("gpt-4o", input).await;

        match result {
            Ok(mut stream) => {
                println!("Success! Stream started.");
                let mut full_text = String::new();
                while let Some(chunk_res) = stream.next().await {
                    match chunk_res {
                        Ok(chunk) => {
                            print!("{}", chunk.content);
                            full_text.push_str(&chunk.content);
                        }
                        Err(e) => println!("Chunk error: {}", e),
                    }
                }
                println!("\nStream finished. Full text: {}", full_text);
                assert!(
                    !full_text.is_empty(),
                    "Streamed content should not be empty"
                );
            }
            Err(e) => panic!("Failed to get stream: {}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_models() {
        let api_key = get_api_key();
        let service = OpenAiService::new(api_key, Duration::from_secs(60));

        let result = service.get_available_models().await;

        match result {
            Ok(Some(models)) => {
                println!("Success! Models: {:?}", models);
                assert!(!models.is_empty(), "Models list should not be empty");
            }
            Ok(None) => panic!("Should return Some(models)"),
            Err(e) => panic!("Failed to get models: {}", e),
        }
    }
}
