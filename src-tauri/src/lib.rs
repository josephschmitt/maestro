mod commands;
mod config;
mod db;

use commands::config::ConfigState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_state =
                ConfigState::load().expect("failed to initialize global config");
            app.manage(config_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_global_config,
            commands::config::set_last_project,
            commands::config::resolve_config,
            commands::projects::create_project,
            commands::projects::get_project,
            commands::projects::list_projects,
            commands::projects::update_project,
            commands::projects::delete_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
