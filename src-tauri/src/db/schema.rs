use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub position: NodePosition,
    pub data: serde_json::Value,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(alias = "in")]
    pub source: Thing,
    #[serde(alias = "out")]
    pub target: Thing,
}
