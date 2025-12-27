use async_trait::async_trait;

pub mod google;
pub mod local;
pub mod openai;

pub type Embedding = Vec<f32>;

#[async_trait]
pub trait TextEmbedding: Send + Sync {
    async fn embed(&self, documents: Vec<String>) -> anyhow::Result<Vec<Embedding>>;
}
