use crate::db::schema::{ChatNode, FlowEdge};
use crate::db::Database;
use tauri::State;

// Defined return structure if needed, or just use tuple
#[tauri::command]
pub async fn save_node_command(
    state: State<'_, Database>,
    node: ChatNode,
) -> Result<ChatNode, String> {
    state.upsert_node(node).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect_edge_command(
    state: State<'_, Database>,
    edge_id: Option<String>,
    source: String,
    target: String,
) -> Result<FlowEdge, String> {
    state
        .connect_nodes(edge_id, source, target)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_board_command(
    state: State<'_, Database>,
) -> Result<(Vec<ChatNode>, Vec<FlowEdge>), String> {
    state.load_graph().await.map_err(|e| e.to_string())
}
