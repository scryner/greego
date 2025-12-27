use crate::embedding::{Embedding, EmbeddingService};
use async_openai::types::embeddings::{CreateEmbeddingRequestArgs, EmbeddingInput};
use async_openai::{config::OpenAIConfig, Client};
use async_trait::async_trait;

pub struct OpenAIEmbedding {
    client: Client<OpenAIConfig>,
}

impl OpenAIEmbedding {
    pub fn new(api_key: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self { client }
    }
}

#[async_trait]
impl EmbeddingService for OpenAIEmbedding {
    async fn embed(&self, model: &str, documents: Vec<String>) -> anyhow::Result<Vec<Embedding>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(model)
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

    async fn get_available_models(&self) -> anyhow::Result<Option<Vec<String>>> {
        Ok(Some(vec![
            "text-embedding-3-small".to_string(),
            "text-embedding-3-large".to_string(),
            "text-embedding-ada-002".to_string(),
        ]))
    }
}
