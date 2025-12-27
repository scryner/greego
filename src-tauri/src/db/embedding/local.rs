use super::{Embedding, TextEmbedding};
use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding as FastTextEmbedding};
use hf_hub::api::tokio::Api;
use std::sync::{Arc, Mutex};

pub struct LocalEmbedding {
    model: Arc<Mutex<FastTextEmbedding>>,
}

impl LocalEmbedding {
    pub async fn new() -> anyhow::Result<Self> {
        // Initialize fastembed, which will find the cached files
        // We use spawn_blocking because try_new is blocking/heavy
        let model = tokio::task::spawn_blocking(move || {
            let mut options = InitOptions::new(EmbeddingModel::EmbeddingGemma300M)
                .with_show_download_progress(true);

            #[cfg(feature = "apple")]
            {
                options = options.with_execution_providers(vec![
                    ort::execution_providers::CoreMLExecutionProvider::default()
                        .with_subgraphs(true)
                        .build(),
                    ort::execution_providers::XNNPACKExecutionProvider::default().build(),
                ]);
            }

            FastTextEmbedding::try_new(options)
        })
        .await??;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
}

#[async_trait]
impl TextEmbedding for LocalEmbedding {
    async fn embed(&self, documents: Vec<String>) -> anyhow::Result<Vec<Embedding>> {
        let model = self.model.clone();

        let embeddings = tokio::task::spawn_blocking(move || {
            let mut guard = model
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock model"))?;
            guard.embed(documents, None)
        })
        .await??;

        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_local() -> anyhow::Result<()> {
        let embedding_provider = LocalEmbedding::new().await?;

        let documents = vec!["Hello, world!".to_string(), "Rust is awesome.".to_string()];
        let embeddings = embedding_provider.embed(documents.clone()).await?;

        assert_eq!(embeddings.len(), documents.len());
        for embedding in embeddings {
            assert!(!embedding.is_empty());
        }

        Ok(())
    }
}
