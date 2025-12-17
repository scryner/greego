use crate::db::schema::{Derives, Node, Sequences};
use crate::db::Database;

use tauri::State;

#[tauri::command]
pub async fn save_node_command(state: State<'_, Database>, node: Node) -> Result<Node, String> {
    state.upsert_node(node).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_canvas_command(
    state: State<'_, Database>,
    canvas_id: String,
) -> Result<(Vec<Node>, Vec<Derives>, Vec<Sequences>), String> {
    let thing = surrealdb::sql::thing(&canvas_id).map_err(|e| e.to_string())?;
    state.load_canvas(thing).await.map_err(|e| e.to_string())
}
