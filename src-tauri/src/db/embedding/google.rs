use super::{Embedding, TextEmbedding};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const BATCH_SIZE: usize = 100;

pub struct GoogleEmbedding {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GoogleEmbedding {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "models/text-embedding-004".to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct BatchEmbedContentsRequest {
    requests: Vec<EmbedContentRequest>,
}

#[derive(Serialize)]
struct EmbedContentRequest {
    model: String,
    content: Content,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct BatchEmbedContentsResponse {
    embeddings: Vec<ContentEmbedding>,
}

#[derive(Deserialize)]
struct ContentEmbedding {
    values: Vec<f32>,
}

#[async_trait]
impl TextEmbedding for GoogleEmbedding {
    async fn embed(&self, documents: Vec<String>) -> anyhow::Result<Vec<Embedding>> {
        let mut all_embeddings = Vec::with_capacity(documents.len());

        for chunk in documents.chunks(BATCH_SIZE) {
            let requests: Vec<EmbedContentRequest> = chunk
                .iter()
                .map(|doc| EmbedContentRequest {
                    model: self.model.clone(),
                    content: Content {
                        parts: vec![Part { text: doc.clone() }],
                    },
                })
                .collect();

            let request_body = BatchEmbedContentsRequest { requests };

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/{}:batchEmbedContents?key={}",
                self.model, self.api_key
            );

            let response = self.client.post(&url).json(&request_body).send().await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!("Gemini API Error: {}", error_text));
            }

            let response_body: BatchEmbedContentsResponse = response.json().await?;

            for embedding in response_body.embeddings {
                all_embeddings.push(embedding.values);
            }
        }

        Ok(all_embeddings)
    }
}
