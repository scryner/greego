use std::path::PathBuf;

use crate::llm::{LlmInput, LlmOutput};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use tauri::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub embedding_id: Option<String>,
    pub reranker_id: Option<String>,
    pub chat_model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasData {
    pub canvas: Canvas,
    pub nodes: Vec<Node>,
    pub derives: Vec<Derives>,
    pub sequences: Vec<Sequences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatNodeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    pub input: LlmInput,
    pub output: Option<LlmOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkSource {
    Question,
    Answer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub content: String,
    pub embedding: Vec<f32>,
    pub node: Thing,
    pub canvas: Thing,
    pub created_at: DateTime<Utc>,
    pub source: ChunkSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum NodeType {
    Chat {
        data: ChatNodeData,
        // embedding field removed in favor of Chunk table
    },
    Link {
        url: Url,
    },
    File {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub position: NodePosition,
    #[serde(rename = "type")]
    pub type_: NodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holds {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(alias = "in")]
    pub canvas: Thing,
    #[serde(alias = "out")]
    pub node: Thing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Derives {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(alias = "in")]
    pub from: Thing,
    #[serde(alias = "out")]
    pub to: Thing,
    pub canvas: Thing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(alias = "in")]
    pub from: Thing,
    #[serde(alias = "out")]
    pub to: Thing,
    pub canvas: Thing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasChunk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(alias = "in")]
    pub node: Thing,
    #[serde(alias = "out")]
    pub chunk: Thing,
}
