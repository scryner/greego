use crate::embedding::EmbeddingService;
use anyhow::Result;
use regex::Regex;

pub struct SemanticChunking {
    model: String,
    threshold: f32,
}

impl SemanticChunking {
    pub fn new(model: String, threshold: f32) -> Self {
        Self { model, threshold }
    }

    pub async fn chunk(&self, text: &str, service: &dyn EmbeddingService) -> Result<Vec<String>> {
        let sentences = self.split_sentences(text);
        if sentences.is_empty() {
            return Ok(vec![]);
        }

        let embeddings = service.embed(&self.model, sentences.clone()).await?;

        if embeddings.len() != sentences.len() {
            return Err(anyhow::anyhow!(
                "Mismatch between sentences count and embeddings count"
            ));
        }

        let mut chunks = Vec::new();
        let mut current_chunk = sentences[0].clone();
        let mut last_embedding = &embeddings[0];

        for i in 1..sentences.len() {
            let sentence = &sentences[i];
            let embedding = &embeddings[i];

            let similarity = self.cosine_similarity(last_embedding, embedding);

            if similarity >= self.threshold {
                current_chunk.push_str(" ");
                current_chunk.push_str(sentence);
                // Keep the last embedding as the reference for the "current topic" logic?
                // Or should we compare with the *last sentence* to detect local shifts?
                // The prompt says: "adjacent two sentences".
                // So we compare `i` and `i-1`.
                // `last_embedding` is `i-1`.
            } else {
                chunks.push(current_chunk);
                current_chunk = sentence.clone();
            }
            last_embedding = embedding;
        }
        chunks.push(current_chunk);

        Ok(chunks)
    }

    fn split_sentences(&self, text: &str) -> Vec<String> {
        // Simple regex-based splitter
        // Matches [.!?] followed by space or end of string.
        let re = Regex::new(r"(?P<sent>.*?[.!?])(?:\s+|$)").unwrap();
        let mut sentences = Vec::new();

        let mut last_end = 0;
        for caps in re.captures_iter(text) {
            if let Some(sent) = caps.name("sent") {
                sentences.push(sent.as_str().trim().to_string());
                last_end = sent.end();
            }
        }

        // Handle any remaining text that might not end with punctuation
        if last_end < text.len() {
            let remainder = text[last_end..].trim();
            if !remainder.is_empty() {
                sentences.push(remainder.to_string());
            }
        }

        // Fallback: if regex didn't find anything but text is not empty (no punctuation)
        if sentences.is_empty() && !text.trim().is_empty() {
            sentences.push(text.trim().to_string());
        }

        sentences
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        use simsimd::SpatialSimilarity;
        match f32::cosine(a, b) {
            Some(distance) => 1.0 - distance as f32,
            None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::Embedding;
    use async_trait::async_trait;

    struct MockEmbeddingService {
        // Map sentence to embedding
        embeddings: std::collections::HashMap<String, Embedding>,
    }

    impl MockEmbeddingService {
        fn new() -> Self {
            Self {
                embeddings: std::collections::HashMap::new(),
            }
        }

        fn add_embedding(&mut self, text: &str, embedding: Embedding) {
            self.embeddings.insert(text.to_string(), embedding);
        }
    }

    #[async_trait]
    impl EmbeddingService for MockEmbeddingService {
        async fn embed(&self, _model: &str, documents: Vec<String>) -> Result<Vec<Embedding>> {
            let mut result = Vec::new();
            for doc in documents {
                if let Some(embedding) = self.embeddings.get(&doc) {
                    result.push(embedding.clone());
                } else {
                    // Return zero vector or error? For test, let's return a default
                    result.push(vec![0.0; 3]);
                }
            }
            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_split_sentences() {
        let chunker = SemanticChunking::new("model".to_string(), 0.8);
        let text = "Hello world. This is a test! Is it working?";
        let sentences = chunker.split_sentences(text);
        assert_eq!(
            sentences,
            vec!["Hello world.", "This is a test!", "Is it working?"]
        );
    }

    #[tokio::test]
    async fn test_split_sentences_no_punctuation() {
        let chunker = SemanticChunking::new("model".to_string(), 0.8);
        let text = "Hello world";
        let sentences = chunker.split_sentences(text);
        assert_eq!(sentences, vec!["Hello world"]);
    }

    #[tokio::test]
    async fn test_chunking() {
        let chunker = SemanticChunking::new("model".to_string(), 0.9); // High threshold
        let mut service = MockEmbeddingService::new();

        // Topic A
        service.add_embedding("Sentence A1.", vec![1.0, 0.0, 0.0]);
        service.add_embedding("Sentence A2.", vec![0.99, 0.05, 0.0]); // Very close to A1

        // Topic B
        service.add_embedding("Sentence B1.", vec![0.0, 1.0, 0.0]); // Orthogonal to A
        service.add_embedding("Sentence B2.", vec![0.0, 0.95, 0.05]); // Close to B1

        let text = "Sentence A1. Sentence A2. Sentence B1. Sentence B2.";

        let chunks = chunker.chunk(text, &service).await.unwrap();

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "Sentence A1. Sentence A2.");
        assert_eq!(chunks[1], "Sentence B1. Sentence B2.");
    }
}
