pub mod commands;
pub mod db;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize DB
            let app_handle = app.handle().clone();
            let db_arc =
                tauri::async_runtime::block_on(async move { Database::init(app_handle).await })
                    .expect("Failed to initialize database");

            // Manage the Database (cloning the struct which holds the Arc)
            app.manage((*db_arc).clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_node_command,
            commands::add_derived_node_command,
            commands::add_sequenced_node_command,
            commands::move_node_position_command,
            commands::delete_node_command,
            commands::load_canvas_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
