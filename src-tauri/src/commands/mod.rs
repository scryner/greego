use crate::db::schema::{Derives, Node, Sequences};
use crate::db::Database;
use crate::llm::LlmService;
use std::sync::Arc;
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
    app_handle: tauri::AppHandle,
    state: State<'_, Database>,
    llm_service: State<'_, Arc<dyn LlmService + Send + Sync>>,
    canvas_id: String,
    prompt: String,
    x: f64,
    y: f64,
    parent_id: Option<String>,
    relation_type: Option<String>,
) -> Result<Vec<Node>, String> {
    use crate::db::schema::{NodePosition, NodeType};
    use crate::llm::{LlmInput, Message, Role};
    use futures::StreamExt;
    use serde_json::json;
    use tauri::Emitter;

    let (tb, id_str) = canvas_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let canvas_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));

    // 1. Create Consolidated Node (User + Assistant Placeholder)
    let assistant_response_placeholder = "";

    let chat_node = Node {
        id: None,
        position: NodePosition { x, y },
        type_: NodeType::Chat {
            value: json!({
                "prompt": prompt,
                "text": assistant_response_placeholder,
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
                .add_sequenced_node(canvas_thing.clone(), parent_thing, chat_node)
                .await
                .map_err(|e| e.to_string())?,
            Some("derive") => state
                .add_derived_node(canvas_thing.clone(), parent_thing, chat_node)
                .await
                .map_err(|e| e.to_string())?,
            _ => state
                .add_derived_node(canvas_thing.clone(), parent_thing, chat_node)
                .await
                .map_err(|e| e.to_string())?,
        }
    } else {
        state
            .add_node(canvas_thing.clone(), chat_node)
            .await
            .map_err(|e| e.to_string())?
    };

    let node_id = saved_node
        .id
        .clone()
        .ok_or("Failed to get node ID")?
        .to_string();
    let prompt_clone = prompt.clone();

    // Spawn background task
    let llm_service = llm_service.inner().clone();
    let app_handle_clone = app_handle.clone();
    let node_id_clone = node_id.clone();
    let _state_clone = state.inner().clone(); // Clone Database for updating later if needed

    // Note: We need to update the node content in DB after streaming is done.
    // Currently `state` (Database) is available.
    // However, `Database` methods are async.

    // We will just stream events for now. Updating DB at the end is good practice.

    tokio::spawn(async move {
        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![], // TODO: Retrieve history from DB based on parent traversal?
            user_input: Message::new_text(Role::User, prompt_clone.clone()),
        };

        match llm_service.chat_stream(input).await {
            Ok(mut stream) => {
                let mut full_text = String::new();
                while let Some(chunk_res) = stream.next().await {
                    if let Ok(chunk) = chunk_res {
                        full_text.push_str(&chunk.content);
                        // Emit event
                        let _ = app_handle_clone.emit(
                            "chat-delta",
                            json!({
                                "node_id": node_id_clone,
                                "content": chunk.content,
                            }),
                        );
                    }
                }

                // Optimize: Update DB with full text
                // We need a method in Database to update node value.
                // Assuming we can just overwrite the node or update specific field.
                // For now, let's just log or emit done.
                let _ = app_handle_clone.emit(
                    "chat-done",
                    json!({
                        "node_id": node_id_clone,
                        "full_text": full_text
                    }),
                );

                // TODO: Implement DB update for persistence of the answer.
                // Since we don't have a direct `update_node` command exposed easily here without Thing,
                // and we have `node_id_clone` string.
                // We can construct Thing and update.
                // let (tb, id) = node_id_clone.split_once(':').unwrap();
                // let thing = surrealdb::sql::Thing::from((tb.to_string(), id.to_string()));
                // But we need to update just the "text" field inside "value" of "type_".
                // That might be complex with current Database struct if not exposed.
                // Postponed for verification step or next iteration.
            }
            Err(e) => {
                let _ = app_handle_clone.emit(
                    "chat-error",
                    json!({
                        "node_id": node_id_clone,
                        "error": e.to_string()
                    }),
                );
            }
        }
    });

    Ok(vec![saved_node])
}
