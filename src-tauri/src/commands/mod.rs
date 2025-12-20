use crate::db::schema::{Derives, Node, Sequences};
use crate::db::Database;

use tauri::State;

#[tauri::command]
pub async fn load_canvas_command(
    state: State<'_, Database>,
    canvas_id: String,
) -> Result<(Vec<Node>, Vec<Derives>, Vec<Sequences>), String> {
    let (tb, id_str) = canvas_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));
    state.load_canvas(thing).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_node_command(
    state: State<'_, Database>,
    node_id: String,
) -> Result<(), String> {
    let (tb, id_str) = node_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));
    state.delete_node(thing).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_node_command(
    state: State<'_, Database>,
    canvas_id: String,
    node: Node,
) -> Result<Node, String> {
    let (tb, id_str) = canvas_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let canvas_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));
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
    let (tb, id_str) = canvas_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let canvas_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));

    let (tb, id_str) = from_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let from_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));
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
    let (tb, id_str) = canvas_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let canvas_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));

    let (tb, id_str) = from_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let from_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));
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
    let (tb, id_str) = node_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let node_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));
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
    x: f64,
    y: f64,
    parent_id: Option<String>,
    relation_type: Option<String>,
) -> Result<Vec<Node>, String> {
    use crate::db::schema::{NodePosition, NodeType};
    use serde_json::json;
    use std::time::Duration;

    let (tb, id_str) = canvas_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let canvas_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));

    // 1. Simulate LLM Delay
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 2. Create Consolidated Node
    // We store both the prompt and the response in a single node
    let assistant_response = "This is a simulated response from the backend.";

    let chat_node = Node {
        id: None,
        position: NodePosition { x, y },
        type_: NodeType::Chat {
            value: json!({
                "prompt": prompt,
                "text": assistant_response,
                "role": "assistant"
            }),
        },
    };

    let saved_node = if let Some(parent_id) = parent_id {
        let (tb, id_str) = parent_id
            .split_once(':')
            .ok_or("Invalid Parent ID format. Expected 'table:id'")?;
        let parent_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));

        match relation_type.as_deref() {
            Some("sequence") => state
                .add_sequenced_node(canvas_thing, parent_thing, chat_node)
                .await
                .map_err(|e| e.to_string())?,
            Some("derive") => state
                .add_derived_node(canvas_thing, parent_thing, chat_node)
                .await
                .map_err(|e| e.to_string())?,
            _ => {
                // Default or fallback if relation type is unknown, treated as standalone for now or error?
                // Let's fallback to derived for safety or standalone.
                // Given the requirement, specific types are needed.
                // If we have parent but no valid type, let's error or default to derived.
                state
                    .add_derived_node(canvas_thing, parent_thing, chat_node)
                    .await
                    .map_err(|e| e.to_string())?
            }
        }
    } else {
        state
            .add_node(canvas_thing, chat_node)
            .await
            .map_err(|e| e.to_string())?
    };

    Ok(vec![saved_node])
}
