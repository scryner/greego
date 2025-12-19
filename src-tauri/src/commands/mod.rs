use crate::db::schema::{Derives, Node, Sequences};
use crate::db::Database;

use tauri::State;

#[tauri::command]
pub async fn load_canvas_command(
    state: State<'_, Database>,
    canvas_id: String,
) -> Result<(Vec<Node>, Vec<Derives>, Vec<Sequences>), String> {
    let thing = surrealdb::sql::thing(&canvas_id).map_err(|e| e.to_string())?;
    state.load_canvas(thing).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_node_command(
    state: State<'_, Database>,
    node_id: String,
) -> Result<(), String> {
    let thing = surrealdb::sql::thing(&node_id).map_err(|e| e.to_string())?;
    state.delete_node(thing).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_node_command(
    state: State<'_, Database>,
    canvas_id: String,
    node: Node,
) -> Result<Node, String> {
    let canvas_thing = surrealdb::sql::thing(&canvas_id).map_err(|e| e.to_string())?;
    state
        .add_node(canvas_thing, node)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_derived_node_command(
    state: State<'_, Database>,
    canvas_id: String,
    from_id: String,
    to_node: Node,
) -> Result<Node, String> {
    let canvas_thing = surrealdb::sql::thing(&canvas_id).map_err(|e| e.to_string())?;
    let from_thing = surrealdb::sql::thing(&from_id).map_err(|e| e.to_string())?;
    state
        .add_derived_node(canvas_thing, from_thing, to_node)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_sequenced_node_command(
    state: State<'_, Database>,
    canvas_id: String,
    from_id: String,
    to_node: Node,
) -> Result<Node, String> {
    let canvas_thing = surrealdb::sql::thing(&canvas_id).map_err(|e| e.to_string())?;
    let from_thing = surrealdb::sql::thing(&from_id).map_err(|e| e.to_string())?;
    state
        .add_sequenced_node(canvas_thing, from_thing, to_node)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn move_node_position_command(
    state: State<'_, Database>,
    node_id: String,
    x: f64,
    y: f64,
) -> Result<Node, String> {
    let node_thing = surrealdb::sql::thing(&node_id).map_err(|e| e.to_string())?;
    state
        .move_node_position(node_thing, x, y)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn invoke_chat_command(
    state: State<'_, Database>,
    canvas_id: String,
    prompt: String,
) -> Result<Vec<Node>, String> {
    use crate::db::schema::{NodePosition, NodeType};
    use serde_json::json;
    use std::time::Duration;

    let canvas_thing = surrealdb::sql::thing(&canvas_id).map_err(|e| e.to_string())?;

    // 1. Simulate LLM Delay
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 2. Create Consolidated Node
    // We store both the prompt and the response in a single node
    let assistant_response = "This is a simulated response from the backend.";

    let chat_node = Node {
        id: None,
        position: NodePosition { x: 0.0, y: 0.0 },
        type_: NodeType::Chat {
            value: json!({
                "prompt": prompt,
                "text": assistant_response,
                "role": "assistant"
            }),
        },
    };

    let saved_node = state
        .add_node(canvas_thing, chat_node)
        .await
        .map_err(|e| e.to_string())?;

    Ok(vec![saved_node])
}
