mod commands;
mod config;
mod db;
mod fs;

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
            commands::statuses::list_statuses,
            commands::statuses::create_status,
            commands::statuses::update_status,
            commands::statuses::delete_status,
            commands::statuses::reorder_statuses,
            commands::cards::create_card,
            commands::cards::get_card,
            commands::cards::update_card,
            commands::cards::delete_card,
            commands::cards::list_cards,
            commands::cards::list_sub_cards,
            commands::cards::move_card,
            commands::cards::reorder_cards,
            commands::questions::create_question,
            commands::questions::list_questions,
            commands::questions::resolve_question,
            commands::questions::unresolve_question,
            commands::questions::delete_question,
            commands::questions::count_unresolved_questions,
            commands::artifacts::create_artifact,
            commands::artifacts::read_artifact,
            commands::artifacts::update_artifact,
            commands::artifacts::delete_artifact,
            commands::artifacts::list_artifacts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
