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
            let db_arc = tauri::async_runtime::block_on(async { Database::init().await })
                .expect("Failed to initialize database");

            // Manage the Database (cloning the struct which holds the Arc)
            app.manage((*db_arc).clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::save_node_command,
            // commands::connect_edge_command,
            commands::load_canvas_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
