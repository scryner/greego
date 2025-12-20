use crate::llm::{ContentPart, LlmInput, LlmOutput, LlmService, Message, Role, TokenUsage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

pub struct AnthropicService {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicService {
    pub fn new(api_key: String, model: String, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            api_key,
            model,
        }
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
    async fn chat_completion(&self, input: LlmInput) -> anyhow::Result<LlmOutput> {
        let mut messages = Vec::new();

        for msg in input.history {
            messages.push(convert_message(msg));
        }
        messages.push(convert_message(input.user_input));

        let system = input.system_prompt;
        let max_tokens = 4096;

        let request_body = CreateMessageRequest {
            model: self.model.clone(),
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
        _input: LlmInput,
    ) -> anyhow::Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = anyhow::Result<crate::llm::LlmStreamChunk>> + Send>,
        >,
    > {
        Err(anyhow::anyhow!("Anthropic stream not implemented yet"))
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
