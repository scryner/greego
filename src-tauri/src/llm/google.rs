use crate::llm::{ContentPart, LlmInput, LlmOutput, LlmService, Message, Role, TokenUsage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct GoogleService {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl GoogleService {
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

// Data structures for Google API

#[derive(Serialize)]
struct GenerateContentRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GoogleContent>,
}

#[derive(Serialize)]
struct GoogleContent {
    role: String,
    parts: Vec<GooglePart>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GooglePart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline_data: Option<GoogleInlineData>,
}

#[derive(Serialize)]
struct GoogleInlineData {
    mime_type: String,
    data: String,
}

#[derive(Deserialize)]
struct GenerateContentResponse {
    candidates: Option<Vec<Candidate>>,
    usage_metadata: Option<GoogleUsage>,
}

#[derive(Deserialize)]
struct Candidate {
    content: GoogleContentResponse,
    // finishReason, safetyRatings, etc. could be added here
}

#[derive(Deserialize)]
struct GoogleContentResponse {
    parts: Option<Vec<GooglePartResponse>>,
}

#[derive(Deserialize)]
struct GooglePartResponse {
    text: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleUsage {
    prompt_token_count: u32,
    candidates_token_count: u32,
    total_token_count: u32,
}

#[async_trait]
impl LlmService for GoogleService {
    async fn chat_completion(&self, input: LlmInput) -> anyhow::Result<LlmOutput> {
        let mut contents = Vec::new();

        for msg in input.history {
            contents.push(convert_message(msg));
        }
        contents.push(convert_message(input.user_input));

        let system_instruction = input.system_prompt.map(|prompt| GoogleContent {
            role: "user".to_string(), // System instructions are technically 'user' role in some contexts or separate field.
            // Actually, for Gemini 1.5 format, system_instruction is a separate field with role 'model' or just content?
            // Checking API spec: 'system_instruction' field, Content type.
            parts: vec![GooglePart {
                text: Some(prompt),
                inline_data: None,
            }],
        });

        let request_body = GenerateContentRequest {
            contents,
            system_instruction,
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self.client.post(&url).json(&request_body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Google API Error: {} - {}", status, text));
        }

        let response_body: GenerateContentResponse = response.json().await?;

        let mut content = Vec::new();
        if let Some(candidates) = response_body.candidates {
            if let Some(first_candidate) = candidates.first() {
                if let Some(ref parts) = first_candidate.content.parts {
                    for part in parts {
                        if let Some(ref text) = part.text {
                            content.push(ContentPart::Text(text.clone()));
                        }
                    }
                }
            }
        }

        let usage = response_body.usage_metadata.map(|u| TokenUsage {
            prompt_tokens: u.prompt_token_count,
            completion_tokens: u.candidates_token_count,
            total_tokens: u.total_token_count,
        });

        Ok(LlmOutput {
            content,
            usage,
            raw: None, // Could capture full response if needed
        })
    }
}

fn convert_message(msg: Message) -> GoogleContent {
    let role = match msg.role {
        Role::System => "user", // Google doesn't strictly have system role in 'contents', mostly 'user' and 'model'
        Role::User => "user",
        Role::Assistant => "model",
        Role::Tool => "function", // Simplification
    }
    .to_string();

    let parts = msg
        .content
        .iter()
        .map(|part| match part {
            ContentPart::Text(t) => GooglePart {
                text: Some(t.clone()),
                inline_data: None,
            },
            ContentPart::ImageUrl(url) => {
                // Heuristic: Check if base64 or url. Gemini InlineData needs base64 without prefix.
                // Assuming url might be data URI.
                if let Some(base64_data) = url.strip_prefix("data:image/") {
                    // Extract mime and data
                    // Format: data:image/png;base64,xxxxxx
                    let parts: Vec<&str> = base64_data.split(";base64,").collect();
                    if parts.len() == 2 {
                        let mime_type = format!("image/{}", parts[0]); // parts[0] is 'png'
                        let data = parts[1].to_string();
                        return GooglePart {
                            text: None,
                            inline_data: Some(GoogleInlineData { mime_type, data }),
                        };
                    }
                }

                // Fallback for logic if strict URL is passed (Gemini usually needs uploaded file URI or inline data).
                // For now, treat simple text as fallback or ignore?
                // Let's just return text representation if we can't parse base64.
                GooglePart {
                    text: Some(format!("[Image: {}]", url)),
                    inline_data: None,
                }
            }
        })
        .collect();

    GoogleContent { role, parts }
}
