use super::{Embedding, TextEmbedding};
use async_openai::types::embeddings::{CreateEmbeddingRequestArgs, EmbeddingInput};
use async_openai::{config::OpenAIConfig, Client};
use async_trait::async_trait;

pub struct OpenAIEmbedding {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIEmbedding {
    pub fn new(api_key: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self {
            client,
            model: "text-embedding-3-small".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait]
impl TextEmbedding for OpenAIEmbedding {
    async fn embed(&self, documents: Vec<String>) -> anyhow::Result<Vec<Embedding>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(EmbeddingInput::StringArray(documents))
            .build()?;

        let response = self.client.embeddings().create(request).await?;

        let embeddings = response
            .data
            .into_iter()
            .map(|embedding| embedding.embedding)
            .collect();

        Ok(embeddings)
    }
}
