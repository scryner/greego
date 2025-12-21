use crate::db::schema::{Derives, Node, Sequences};
use crate::db::Database;
// use crate::llm::LlmService; // Removed
use crate::llm::LlmServiceManager; // Added
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock; // Added

pub mod llm; // Register the new module

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
    llm_service: State<'_, Arc<RwLock<LlmServiceManager>>>, // Changed type
    canvas_id: String,
    prompt: String,
    model: String,
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
    let llm_manager_arc = llm_service.inner().clone(); // Clone Arc<RwLock<Manager>>
    let app_handle_clone = app_handle.clone();
    let node_id_clone = node_id.clone();
    let _state_clone = state.inner().clone();

    tokio::spawn(async move {
        let input = LlmInput {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            history: vec![], // TODO: Retrieve history from DB based on parent traversal?
            user_input: Message::new_text(Role::User, prompt_clone.clone()),
        };

        // Acquire read lock and call stream
        let manager_guard = llm_manager_arc.read().await;
        // The stream must not outlive the guard?
        // chat_stream returns a Pin<Box<Stream + Send>>.
        // Does expected stream lifetime depend on &self?
        // LlmServiceManager::chat_stream signature:
        // fn chat_stream(&self, ...) -> ... Pin<Box<dyn Stream ... >>
        // Usually, if the stream holds reference to self (manager), then we have a problem because guard is dropped.
        // Let's check LlmServiceManager::chat_stream impl.
        // It gets model from HashMap.
        // It calls service.chat_stream.
        // Service.chat_stream returns a stream.
        // If the service's stream owns the Future/Stream, independent of &self, it works.
        // Most HTTP client (reqwest) streams are independent of the client if client is cloned or internally ref-counted (reqwest::Client is).
        // Our services hold reqwest::Client which is cheap to clone or internally Arc.
        // BUT `LlmModel` is inside `HashMap`.
        // `manager.services.get` returns reference.
        // `service.chat_stream` is called on that reference.
        // If `service.chat_stream` returns a Future/Stream that captures `&self` (the service), then it captures reference to Manager's map value.
        // Which is tied to `manager_guard`.
        // So `stream` cannot outlive `manager_guard`.
        // If we iterate stream inside this block, it is fine!
        // We just need to make sure we keep the guard until stream is done.

        match manager_guard.chat_stream(&model, input).await {
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

                let _ = app_handle_clone.emit(
                    "chat-done",
                    json!({
                        "node_id": node_id_clone,
                        "full_text": full_text
                    }),
                );
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
        // Guard is dropped here.
    });

    Ok(vec![saved_node])
}
