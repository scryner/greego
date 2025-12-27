use async_trait::async_trait;

pub mod google;
pub mod local;
pub mod openai;

pub type Embedding = Vec<f32>;

#[async_trait]
pub trait TextEmbedding: Send + Sync {
    async fn embed(&self, documents: Vec<String>) -> anyhow::Result<Vec<Embedding>>;
}
#[derive(Debug, Clone)]
pub struct RerankResult {
    pub index: usize,
    pub score: f32,
    pub document: Option<String>,
}

#[async_trait]
pub trait TextReranking: Send + Sync {
    async fn rerank(
        &self,
        query: &str,
        documents: Vec<String>,
    ) -> anyhow::Result<Vec<RerankResult>>;
}
