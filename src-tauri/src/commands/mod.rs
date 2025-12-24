use crate::db::schema::{Derives, Node, Sequences};
use crate::db::Database;
// use crate::llm::LlmService; // Removed
use crate::llm::LlmServiceManager; // Added
use log::error;
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
    llm_service: State<'_, Arc<RwLock<LlmServiceManager>>>,
    canvas_id: String,
    prompt: String,
    model_id: String,
    x: f64,
    y: f64,
    parent_id: Option<String>,
    relation_type: Option<String>,
) -> Result<Vec<Node>, String> {
    use crate::db::schema::{ChatNodeData, NodePosition, NodeType};
    use crate::llm::{LlmInput, LlmOutput, Message, Role};
    use futures::StreamExt;
    use tauri::Emitter;

    let (tb, id_str) = canvas_id
        .split_once(':')
        .ok_or("Invalid ID format. Expected 'table:id'")?;
    let canvas_thing = surrealdb::sql::Thing::from((tb.to_string(), id_str.to_string()));

    // 1. Create Consolidated Node (User + Assistant Placeholder)
    let input = LlmInput {
        system_prompt: Some("You are a helpful assistant.".to_string()),
        history: vec![], // TODO: Retrieve history from DB based on parent traversal?
        user_input: Message::new_text(Role::User, prompt.clone()),
    };

    let chat_node = Node {
        id: None,
        position: NodePosition { x, y },
        type_: NodeType::Chat {
            data: ChatNodeData {
                input: input.clone(),
                output: None,
            },
            model_id: Some(model_id.clone()),
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

    // Convert string ID back to Thing for operations
    let (tb_node, id_node) = node_id.split_once(':').unwrap();
    let node_thing = surrealdb::sql::Thing::from((tb_node.to_string(), id_node.to_string()));

    // Spawn background task
    let llm_manager_arc = llm_service.inner().clone(); // Clone Arc<RwLock<Manager>>
    let app_handle_clone = app_handle.clone();
    let node_id_clone = node_id.clone();

    // Clone client for background task
    let db_client = state.client.clone();
    let node_thing_clone = node_thing.clone();

    println!(
        "Starting chat background task for node_id: {}",
        node_id_clone
    );

    tokio::spawn(async move {
        // Acquire read lock and call stream
        let manager_guard = llm_manager_arc.read().await;

        // Parse service_id and model_name
        // Assuming format is "service_name/model_name"
        let (service_id, model_name) = match model_id.split_once('/') {
            Some((s, m)) => (s, m),
            None => (model_id.as_str(), ""),
        };

        println!(
            "Requesting chat stream: service_id='{}', model_name='{}'",
            service_id, model_name
        );

        match manager_guard
            .chat_stream(service_id, model_name, input)
            .await
        {
            Ok(mut stream) => {
                let mut full_text = String::new();
                while let Some(chunk_res) = stream.next().await {
                    if let Ok(chunk) = chunk_res {
                        full_text.push_str(&chunk.content);
                        // Emit event
                        let _ = app_handle_clone.emit(
                            "chat-delta",
                            serde_json::json!({
                                "node_id": node_id_clone,
                                "content": chunk.content,
                            }),
                        );
                    } else {
                        error!("Error receiving chunk");
                    }
                }

                let _ = app_handle_clone.emit(
                    "chat-done",
                    serde_json::json!({
                        "node_id": node_id_clone,
                        "full_text": full_text
                    }),
                );

                // Persist the full output
                use crate::llm::ContentPart; // Ensure these are available if needed or just use LlmOutput
                let output = LlmOutput {
                    content: vec![ContentPart::Text(full_text)],
                    usage: None,
                    raw: None,
                };

                use crate::db::operation;
                if let Err(e) =
                    operation::update_chat_node_output(&db_client, node_thing_clone, output).await
                {
                    error!("Failed to persist chat output: {}", e);
                }
            }
            Err(e) => {
                error!("Error starting chat stream: {}", e);
                let _ = app_handle_clone.emit(
                    "chat-error",
                    serde_json::json!({
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
