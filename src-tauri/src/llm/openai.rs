use crate::llm::{ContentPart, LlmInput, LlmOutput, LlmService, Message, Role, TokenUsage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

pub struct OpenAiCompatibleService {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

impl OpenAiCompatibleService {
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        model: String,
        timeout: Duration,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
        }
    }
}

#[derive(Debug, Serialize)]
struct CheckCompletionRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    // stream: bool, // Future support
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: Value, // String or Array of ContentPart
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
    #[serde(flatten)]
    extra: Value,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    // role: String, // usually assistant
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LlmService for OpenAiCompatibleService {
    async fn chat_completion(&self, input: LlmInput) -> anyhow::Result<LlmOutput> {
        let mut messages = Vec::new();

        if let Some(system_prompt) = input.system_prompt {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: Value::String(system_prompt),
            });
        }

        for msg in input.history {
            messages.push(convert_message(msg));
        }

        messages.push(convert_message(input.user_input));

        let request_body = CheckCompletionRequest {
            model: self.model.clone(),
            messages,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let mut request_builder = self.client.post(&url).json(&request_body);

        if let Some(ref key) = self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("LLM API Error: {} - {}", status, text));
        }

        let response_body: ChatCompletionResponse = response.json().await?;

        let content = if let Some(choice) = response_body.choices.first() {
            if let Some(ref text) = choice.message.content {
                vec![ContentPart::Text(text.clone())]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        Ok(LlmOutput {
            content,
            usage: response_body.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            raw: Some(response_body.extra),
        })
    }
}

fn convert_message(msg: Message) -> OpenAiMessage {
    let role = match msg.role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
    .to_string();

    // Check if it's simple text or multimodal
    let is_simple_text = msg.content.len() == 1 && matches!(msg.content[0], ContentPart::Text(_));

    let content = if is_simple_text {
        if let ContentPart::Text(ref t) = msg.content[0] {
            Value::String(t.clone())
        } else {
            // Should not happen due to check above
            Value::String("".to_string())
        }
    } else {
        let parts: Vec<Value> = msg
            .content
            .iter()
            .map(|p| match p {
                ContentPart::Text(t) => json!({ "type": "text", "text": t }),
                ContentPart::ImageUrl(url) => {
                    json!({ "type": "image_url", "image_url": { "url": url } })
                }
            })
            .collect();
        Value::Array(parts)
    };

    OpenAiMessage { role, content }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    #[ignore]
    async fn test_lmstudio_completion() {
        let service = OpenAiCompatibleService::new(
            "http://192.168.0.130:1234/v1".to_string(),
            Some("lm-studio".to_string()),
            "gpt-oss-120b".to_string(),
            Duration::from_secs(60),
        );

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, who are you?"),
        };

        let result = service.chat_completion(input).await;

        match result {
            Ok(output) => {
                println!("Success! Output: {:?}", output);
                assert!(!output.content.is_empty(), "Content should not be empty");
            }
            Err(e) => panic!("Failed to get completion: {}", e),
        }
    }
}
