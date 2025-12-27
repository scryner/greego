pub mod chunking;
pub mod provider;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use std::collections::HashMap;

pub type Embedding = Vec<f32>;

#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn embed(&self, model: &str, documents: Vec<String>) -> Result<Vec<Embedding>>;

    async fn get_available_models(&self) -> Result<Option<Vec<String>>> {
        Ok(None)
    }
}

pub struct EmbeddingServiceManager {
    services: HashMap<String, Box<dyn EmbeddingService>>,
}

impl Default for EmbeddingServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingServiceManager {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn add_service(&mut self, name: String, service: Box<dyn EmbeddingService>) {
        self.services.insert(name.clone(), service);
        let all_services: Vec<_> = self.services.keys().collect();
        debug!("added Embedding service: {} / {:?}", name, all_services);
    }

    pub fn delete_service(&mut self, name: &str) {
        self.services.remove(name);
        let all_services: Vec<_> = self.services.keys().collect();
        debug!("removed Embedding service: {} / {:?}", name, all_services);
    }

    pub fn list_services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    pub async fn get_all_available_models(&self) -> Vec<String> {
        let mut all_models = Vec::new();
        for (service_name, service) in &self.services {
            match service.get_available_models().await {
                Ok(Some(models)) => {
                    for model in models {
                        all_models.push(format!("{}/{}", service_name, model));
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    log::error!("Failed to get models for {}: {}", service_name, e);
                }
            }
        }
        all_models.sort();
        debug!("get_all_available_models: {:?}", all_models);
        all_models
    }

    pub async fn embed(
        &self,
        service_name: &str,
        model_name: &str,
        documents: Vec<String>,
    ) -> Result<Vec<Embedding>> {
        let service = self
            .services
            .get(service_name)
            .ok_or(anyhow::anyhow!("Service not found"))?;
        service.embed(model_name, documents).await
    }
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
