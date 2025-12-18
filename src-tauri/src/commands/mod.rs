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

    // 1. Create User Node
    let user_node = Node {
        id: None,
        position: NodePosition { x: 0.0, y: 0.0 }, // Frontend should position it, but for now 0,0 or we need to accept position
        type_: NodeType::Chat {
            value: json!({ "text": prompt, "role": "user" }),
        },
    };

    // We need to determine position. Ideally frontend sends it.
    // For now, let's assume 0,0 is okay or maybe we should take x,y as args.
    // The prompt implies we just "add node... chat prompt... invoke backend".
    // Let's stick to 0,0 and let frontend handle layout or accept it in args if needed.
    // Wait, the user said "Add node when + button pressed... input screen".
    // Maybe the 'invoke' only happens AFTER user types?
    // "Actual backend should be saved when chat prompt is pressed and llm service response received".
    // So the frontend creates a temporary node?
    // If I return the new nodes, the frontend can replace the temp one.

    let saved_user_node = state
        .add_node(canvas_thing.clone(), user_node)
        .await
        .map_err(|e| e.to_string())?;

    // 2. Simulate LLM Delay
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 3. Create Bot Node
    let bot_node = Node {
        id: None,
        position: NodePosition { x: 0.0, y: 0.0 }, // Should ideally range it?
        type_: NodeType::Chat {
            value: json!({ "text": "This is a simulated response from the backend.", "role": "assistant" }),
        },
    };

    // We can't easily position it relative to user node without reading user node position (which is 0,0).
    // Let's just add it.

    let saved_bot_node = state
        .add_derived_node(canvas_thing, saved_user_node.id.clone().unwrap(), bot_node)
        .await
        .map_err(|e| e.to_string())?;

    Ok(vec![saved_user_node, saved_bot_node])
}
