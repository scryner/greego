use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use tauri::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum NodeType {
    Chat { value: serde_json::Value },
    Link { url: Url },
    File { path: PathBuf },
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
