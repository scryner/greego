pub mod commands;
pub mod db;
pub mod llm;

use db::Database;
use llm::{openai::OpenAiCompatibleService, LlmService};
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .with_colors(ColoredLevelConfig::default())
                        .level(log::LevelFilter::Debug)
                        .build(),
                )?;
            }

            // Initialize DB
            let app_handle = app.handle().clone();
            // Default to InMemory for now, as requested.
            // TODO: Make this configurable via config file or env var if needed.
            let config = db::DatabaseConfig::InMemory;

            let db_arc =
                tauri::async_runtime::block_on(
                    async move { Database::init(app_handle, config).await },
                )
                .expect("Failed to initialize database");

            // Manage the Database (cloning the struct which holds the Arc)
            app.manage((*db_arc).clone());

            // Initialize LLM Service
            // TODO: Load from config
            let llm_service = OpenAiCompatibleService::new(
                "http://192.168.0.130:1234/v1".to_string(), // Default for dev?
                Some("lm-studio".to_string()),
                "gpt-oss-120b".to_string(),
                Duration::from_secs(60),
            );
            let llm_service_arc: Arc<dyn LlmService + Send + Sync> = Arc::new(llm_service);
            app.manage(llm_service_arc);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_node_command,
            commands::add_derived_node_command,
            commands::add_sequenced_node_command,
            commands::move_node_position_command,
            commands::delete_node_command,
            commands::load_canvas_command,
            commands::invoke_chat_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
