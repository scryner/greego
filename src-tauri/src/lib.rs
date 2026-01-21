pub mod commands;
pub mod db;
pub mod embedding;
pub mod llm;

use db::Database;
// use llm::{openai_compatible::OpenAiCompatibleService, LlmService}; // Removed
use embedding::EmbeddingServiceManager; // Added
use llm::LlmServiceManager; // Added
use std::sync::Arc;
// use std::time::Duration; // Removed unused
use tauri::Manager;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tokio::sync::RwLock; // Added

#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub async fn run() {
    let mut builder = tauri::Builder::default();

    if cfg!(debug_assertions) {
        builder = builder.plugin(
            tauri_plugin_log::Builder::default()
                .with_colors(ColoredLevelConfig::default())
                .level(log::LevelFilter::Debug)
                .build(),
        );
    }

    let app = builder
        .invoke_handler(tauri::generate_handler![
            commands::add_node_command,
            commands::add_canvas_command,
            commands::list_canvas_command,
            commands::add_derived_node_command,
            commands::add_sequenced_node_command,
            commands::move_node_position_command,
            commands::delete_node_command,
            commands::load_canvas_command,
            commands::invoke_chat_command,
            commands::llm::add_llm_service,
            commands::llm::delete_llm_service,
            commands::llm::list_llm_services,
            commands::llm::get_llm_available_models,
            commands::update_canvas_chat_model_command,
            commands::unified_query_command,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // Initialize DB Config
    let config = db::DatabaseConfig {
        connection: db::DatabaseConnectionConfig::InMemory,
        default_first_canvas_title: "New Canvas".to_string(),
    };

    // Initialize LLM Service Manager
    let llm_manager = LlmServiceManager::new();
    let llm_manager_arc = Arc::new(RwLock::new(llm_manager));
    app.manage(llm_manager_arc);

    // Initialize Embedding Service Manager
    let mut embedding_manager = EmbeddingServiceManager::new();
    embedding_manager.initialize_local().await;
    let embedding_manager_arc = Arc::new(RwLock::new(embedding_manager));
    app.manage(embedding_manager_arc.clone());

    // Initialize Database
    let app_handle = app.handle().clone();
    let db_arc = Database::init(app_handle, config, embedding_manager_arc)
        .await
        .expect("Failed to initialize database");
    app.manage((*db_arc).clone());

    app.run(|_, _| {});
}
