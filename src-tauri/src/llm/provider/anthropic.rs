use crate::llm::{ContentPart, LlmInput, LlmOutput, LlmService, Message, Role, TokenUsage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

pub struct AnthropicService {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicService {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client, api_key }
    }
}

// Anthropic API Structs
#[derive(Serialize)]
struct CreateMessageRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContent {
    Text { text: String },
    Image { source: AnthropicImageSource },
}

#[derive(Serialize)]
struct AnthropicImageSource {
    #[serde(rename = "type")]
    source_type: String, // "base64"
    media_type: String, // "image/jpeg", etc.
    data: String,
}

#[derive(Deserialize)]
struct CreateMessageResponse {
    content: Vec<AnthropicContentResponse>,
    usage: AnthropicUsage,
    // id, type, role, model, stop_reason, etc.
}

#[derive(Deserialize)]
struct AnthropicContentResponse {
    text: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl LlmService for AnthropicService {
    async fn chat_completion(&self, model: &str, input: LlmInput) -> anyhow::Result<LlmOutput> {
        let mut messages = Vec::new();

        for msg in input.history {
            messages.push(convert_message(msg));
        }
        messages.push(convert_message(input.user_input));

        let system = input.system_prompt;
        let max_tokens = 4096;

        let request_body = CreateMessageRequest {
            model: model.to_string(),
            messages,
            system,
            max_tokens,
        };

        let url = "https://api.anthropic.com/v1/messages";

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Anthropic API Error: {} - {}",
                status,
                text
            ));
        }

        let response_body: CreateMessageResponse = response.json().await?;

        let mut content = Vec::new();
        for part in response_body.content {
            content.push(ContentPart::Text(part.text));
        }

        Ok(LlmOutput {
            content,
            usage: Some(TokenUsage {
                prompt_tokens: response_body.usage.input_tokens,
                completion_tokens: response_body.usage.output_tokens,
                total_tokens: response_body.usage.input_tokens + response_body.usage.output_tokens,
            }),
            raw: None,
        })
    }

    async fn chat_stream(
        &self,
        model: &str,
        input: LlmInput,
    ) -> anyhow::Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = anyhow::Result<crate::llm::LlmStreamChunk>> + Send>,
        >,
    > {
        let mut messages = Vec::new();

        for msg in input.history {
            messages.push(convert_message(msg));
        }
        messages.push(convert_message(input.user_input));

        let system = input.system_prompt;
        let max_tokens = 4096;

        let mut request_body = serde_json::to_value(CreateMessageRequest {
            model: model.to_string(),
            messages,
            system,
            max_tokens,
        })?;

        // Add stream: true
        if let Some(obj) = request_body.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
        }

        let url = "https://api.anthropic.com/v1/messages";

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Anthropic API Error: {} - {}",
                status,
                text
            ));
        }

        let stream = response.bytes_stream();
        let stream = futures::StreamExt::map(stream, |chunk_result| {
            chunk_result.map_err(|e| anyhow::anyhow!("Stream error: {}", e))
        });

        // SSE Parser for Anthropic
        let stream = async_stream::try_stream! {
            let mut buffer = String::new();

            for await chunk in stream {
                let bytes = chunk?;
                let text = String::from_utf8_lossy(&bytes);
                buffer.push_str(&text);

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim();
                    let line_content = line.to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line_content.is_empty() {
                        continue;
                    }

                    if let Some(data) = line_content.strip_prefix("data: ") {
                         if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                             if let Some(event_type) = json.get("type").and_then(|t| t.as_str()) {
                                 match event_type {
                                     "content_block_delta" => {
                                         if let Some(delta) = json.get("delta") {
                                             if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                                 yield crate::llm::LlmStreamChunk {
                                                     content: text.to_string(),
                                                     usage: None,
                                                 };
                                             }
                                         }
                                     }
                                     "message_stop" => {
                                         // Stream done
                                     }
                                     _ => {}
                                 }
                             }
                         }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_api_key() -> String {
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set")
    }

    #[tokio::test]
    #[ignore]
    async fn test_anthropic_completion() {
        let api_key = get_api_key();
        let service = AnthropicService::new(api_key, Duration::from_secs(30));

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, tell me a short joke."),
        };

        let result = service.chat_completion("claude-haiku-4-5", input).await;

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
    async fn test_anthropic_stream() {
        use futures::StreamExt;

        let api_key = get_api_key();
        let service = AnthropicService::new(api_key, Duration::from_secs(30));

        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello, tell me a short joke."),
        };

        let result = service.chat_stream("claude-haiku-4-5", input).await;

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
}

fn convert_message(msg: Message) -> AnthropicMessage {
    let role = match msg.role {
        Role::System => "user", // Should not happen in messages for Anthropic (system is separate param)
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "user", // Placeholder, tools not fully implemented here
    }
    .to_string();

    let content = msg
        .content
        .iter()
        .map(|part| match part {
            ContentPart::Text(t) => AnthropicContent::Text { text: t.clone() },
            ContentPart::ImageUrl(url) => {
                // Heuristic: Check for base64 data URI
                if let Some(base64_config) = parse_data_uri(url) {
                    AnthropicContent::Image {
                        source: AnthropicImageSource {
                            source_type: "base64".to_string(),
                            media_type: base64_config.0,
                            data: base64_config.1,
                        },
                    }
                } else {
                    // Fallback since Anthropic only supports base64 images in 'image' block currently (no direct URL)
                    AnthropicContent::Text {
                        text: format!("[Image: {}]", url),
                    }
                }
            }
        })
        .collect();

    AnthropicMessage { role, content }
}

fn parse_data_uri(uri: &str) -> Option<(String, String)> {
    if let Some(rest) = uri.strip_prefix("data:") {
        if let Some(idx) = rest.find(";base64,") {
            let mime_type = rest[..idx].to_string();
            let data = rest[idx + 8..].to_string(); // 8 = len(";base64,")
            return Some((mime_type, data));
        }
    }
    None
}
