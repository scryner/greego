use super::{Embedding, RerankResult, TextEmbedding, TextReranking};
use async_trait::async_trait;
use fastembed::{
    EmbeddingModel, InitOptions, RerankInitOptions, RerankerModel,
    TextEmbedding as FastTextEmbedding, TextRerank,
};

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
                        .with_compute_units(
                            ort::execution_providers::coreml::CoreMLComputeUnits::CPUAndNeuralEngine,
                        )
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

pub struct LocalReranking {
    model: Arc<Mutex<TextRerank>>,
}

impl LocalReranking {
    pub async fn new() -> anyhow::Result<Self> {
        let model = tokio::task::spawn_blocking(move || {
            let mut options = RerankInitOptions::new(RerankerModel::BGERerankerV2M3)
                .with_show_download_progress(true);

            #[cfg(feature = "apple")]
            {
                options = options.with_execution_providers(vec![
                    ort::execution_providers::CoreMLExecutionProvider::default()
                        .with_compute_units(
                            ort::execution_providers::coreml::CoreMLComputeUnits::CPUAndNeuralEngine,
                        )
                        .with_subgraphs(true)
                        .build(),
                    ort::execution_providers::XNNPACKExecutionProvider::default().build(),
                ]);
            }

            TextRerank::try_new(options)
        })
        .await??;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
}

#[async_trait]
impl TextReranking for LocalReranking {
    async fn rerank(
        &self,
        query: &str,
        documents: Vec<String>,
    ) -> anyhow::Result<Vec<RerankResult>> {
        let model = self.model.clone();
        let query = query.to_string();

        let results = tokio::task::spawn_blocking(move || {
            let mut guard = model
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock model"))?;
            let documents_refs: Vec<&str> = documents.iter().map(|s| s.as_str()).collect();
            guard.rerank::<&str>(&query, documents_refs, true, None)
        })
        .await??;

        let results = results
            .into_iter()
            .map(|r| RerankResult {
                index: r.index,
                score: r.score,
                document: r.document,
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_embedding_local() -> anyhow::Result<()> {
        let embedding_provider = LocalEmbedding::new().await?;

        let documents = vec!["Hello, world!".to_string(), "Rust is awesome.".to_string()];
        let embeddings = embedding_provider.embed(documents.clone()).await?;

        assert_eq!(embeddings.len(), documents.len());
        for (i, embedding) in embeddings.iter().enumerate() {
            assert!(!embedding.is_empty());
            println!("Document: \"{}\"", documents[i]);
            println!(
                "Embedding (first 5 dims): {:?}",
                &embedding[..5.min(embedding.len())]
            );
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_reranking_local() -> anyhow::Result<()> {
        let reranker = LocalReranking::new().await?;
        let results = reranker
            .rerank("panda", vec!["hi".into(), "panda is bear".into()])
            .await?;
        assert!(!results.is_empty());

        let top_result = &results[0];
        assert_eq!(top_result.index, 1);
        assert!(top_result.document.is_some());
        assert_eq!(top_result.document.as_deref(), Some("panda is bear"));

        println!("Query: \"panda\"");
        for result in &results {
            println!(
                "Index: {}, Score: {}, Document: {:?}",
                result.index, result.score, result.document
            );
        }

        Ok(())
    }
}
