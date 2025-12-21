use crate::llm::openai_compatible::OpenAiCompatibleService;
use crate::llm::{LlmInput, LlmOutput, LlmService, LlmStreamChunk};
use async_trait::async_trait;
use futures::Stream;
use serde::Deserialize;
use std::pin::Pin;
use std::time::Duration;

pub struct OllamaService {
    inner: OpenAiCompatibleService,
}

impl OllamaService {
    pub fn new(base_url: String, timeout: Duration) -> Self {
        // Ollama usually doesn't need an API key, but OpenAiCompatibleService supports it.
        // We pass None for now as it's optional in Ollama context usually.
        Self {
            inner: OpenAiCompatibleService::new(base_url, None, timeout),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ListModelsResponse {
    models: Vec<ModelData>,
}

#[derive(Debug, Deserialize)]
struct ModelData {
    name: String,
}

#[async_trait]
impl LlmService for OllamaService {
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
        // Construct the URL for /api/tags
        // self.inner.base_url is trimmed of trailing slash but might end in /v1 if user configured it for OpenAI compatibility.
        // But for Ollama API, it is usually just the base URL (e.g. http://localhost:11434).
        // If the user provided "http://localhost:11434/v1", we might need to strip "/v1".

        let base = self.inner.base_url.trim_end_matches('/');
        let url = if base.ends_with("/v1") {
            format!("{}/api/tags", base.trim_end_matches("/v1"))
        } else {
            format!("{}/api/tags", base)
        };

        let response = self.inner.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama API Error: {} - {}", status, text));
        }

        let response_body: ListModelsResponse = response.json().await?;
        let models = response_body.models.into_iter().map(|m| m.name).collect();

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
    async fn test_ollama_completion() {
        let service = OllamaService::new(
            "http://localhost:11434/v1".to_string(),
            Duration::from_secs(60),
        );

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, who are you?"),
        };

        // Pass model explicitely
        let result = service.chat_completion("gemma3:4b-it-qat", input).await;

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
    async fn test_ollama_stream() {
        use futures::StreamExt;

        let service = OllamaService::new(
            "http://localhost:11434/v1".to_string(),
            Duration::from_secs(60),
        );

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, who are you?"),
        };

        // Pass model explicitely
        let result = service.chat_stream("gemma3:4b-it-qat", input).await;

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
    async fn test_ollama_models() {
        let service = OllamaService::new(
            "http://localhost:11434/v1".to_string(),
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
