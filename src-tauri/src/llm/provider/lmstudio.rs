use crate::llm::openai_compatible::OpenAiCompatibleService;
use crate::llm::{LlmInput, LlmOutput, LlmService, LlmStreamChunk};
use async_trait::async_trait;
use futures::Stream;
use serde::Deserialize;
use std::pin::Pin;
use std::time::Duration;

pub struct LMStudioService {
    inner: OpenAiCompatibleService,
}

impl LMStudioService {
    pub fn new(base_url: String, api_key: Option<String>, timeout: Duration) -> Self {
        Self {
            inner: OpenAiCompatibleService::new(base_url, api_key, timeout),
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
impl LlmService for LMStudioService {
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
        // Construct the URL for /api/v0/models
        // self.inner.base_url is expected to be like "http://localhost:1234/v1"
        // We need "http://localhost:1234/api/v0/models"

        let base = self.inner.base_url.trim_end_matches('/');
        let url = if base.ends_with("/v1") {
            format!("{}/api/v0/models", base.trim_end_matches("/v1"))
        } else {
            // Fallback or assume base is root?
            // If base is "http://localhost:1234", then appending /api/v0/models works.
            // But if it is "http://localhost:1234/v1" we handled it.
            // Let's safe-guard:
            format!("{}/api/v0/models", base)
        };

        // Note: The logic above might be slightly brittle if user enters "v1/" or similar variations,
        // but OpenAiCompatibleService trims end slashes.
        // If the user inputs "http://localhost:1234", OpenAiCompatibleService appends "/chat/completions",
        // wait, OpenAiCompatibleService expects the base url to extend to the point where "/chat/completions" is appended.
        // Usually OpenAI base is "https://api.openai.com/v1".
        // So for LMStudio it is "http://localhost:1234/v1".

        let mut request_builder = self.inner.client.get(&url);

        if let Some(ref key) = self.inner.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            // It might be possible that the service doesn't support this endpoint or is different version.
            // Returns specific error or just None?
            // The requirement says "Result<Option<Vec<String>>>".
            // If API fails, maybe return Err?
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("LMStudio API Error: {} - {}", status, text));
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
    use std::time::Duration;

    #[tokio::test]
    #[ignore]
    async fn test_lmstudio_completion() {
        let service = LMStudioService::new(
            "http://192.168.0.130:1234/v1".to_string(),
            Some("lm-studio".to_string()),
            Duration::from_secs(60),
        );

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, who are you?"),
        };

        // Pass model explicitely
        let result = service.chat_completion("gpt-oss-120b", input).await;

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
    async fn test_lmstudio_stream() {
        use futures::StreamExt;

        let service = LMStudioService::new(
            "http://192.168.0.130:1234/v1".to_string(),
            Some("lm-studio".to_string()),
            Duration::from_secs(60),
        );

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, who are you?"),
        };

        // Pass model explicitely
        let result = service.chat_stream("gpt-oss-120b", input).await;

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
    async fn test_lmstudio_models() {
        let service = LMStudioService::new(
            "http://192.168.0.130:1234/v1".to_string(),
            Some("lm-studio".to_string()),
            Duration::from_secs(60),
        );

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
