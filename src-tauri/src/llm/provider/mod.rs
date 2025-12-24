pub mod anthropic;
pub mod config;
pub mod custom;
pub mod google;
pub mod lmstudio;
pub mod ollama;
pub mod openai;
pub mod openai_compatible;

#[cfg(feature = "apple")]
pub mod apple;
