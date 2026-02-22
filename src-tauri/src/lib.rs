mod commands;
mod config;
mod db;
pub mod executor;
mod fs;
pub mod ipc;

use std::sync::Arc;

use commands::config::ConfigState;
use executor::monitor::start_pid_monitor;
use executor::reattach::startup_scan;
use executor::AgentRegistry;
use ipc::server::IpcServer;
use tauri::Manager;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_state =
                ConfigState::load().expect("failed to initialize global config");

            let base_path = config_state
                .with_config(|c| Ok(c.resolve_base_path()))
                .expect("failed to resolve base path");

            let registry = Arc::new(AgentRegistry::new());

            let scan_result = startup_scan(app.handle(), &base_path);
            for ws in &scan_result.reattached {
                eprintln!(
                    "[startup] Re-attached to workspace {} (pid {})",
                    ws.workspace_id, ws.pid
                );
            }
            for ws in &scan_result.failed {
                eprintln!(
                    "[startup] Workspace {} marked as failed (no live process)",
                    ws.workspace_id
                );
            }

            start_pid_monitor(
                app.handle().clone(),
                Arc::clone(&registry),
                base_path,
            );

            app.manage(config_state);
            app.manage(registry);
            app.manage(Arc::new(IpcServer::new()));

            #[cfg(target_os = "macos")]
            for (_, window) in app.webview_windows() {
                let _ = apply_vibrancy(&window, NSVisualEffectMaterial::Sidebar, None, None);
            }

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
            commands::directories::add_linked_directory,
            commands::directories::remove_linked_directory,
            commands::directories::list_linked_directories,
            commands::conversations::create_conversation,
            commands::conversations::list_conversations,
            commands::conversations::create_message,
            commands::conversations::list_messages,
            commands::conversations::count_conversation_messages,
            commands::agent::launch_agent,
            commands::agent::send_agent_input,
            commands::agent::stop_agent,
            commands::agent::resume_agent,
            commands::agent::list_workspaces,
            commands::agent::get_workspace,
            commands::agent::list_running_workspaces,
            commands::agent::stop_all_agents,
            commands::agent::archive_card_workspaces,
            commands::worktrees::generate_branch_name,
            commands::worktrees::create_worktree,
            commands::worktrees::check_worktree_exists,
            commands::worktrees::get_card_worktree,
            commands::worktrees::get_claude_worktree_path,
            commands::ipc::start_ipc_server,
            commands::ipc::stop_ipc_server,
            commands::review::get_changed_files,
            commands::review::get_file_diff,
            commands::review::send_back_card,
            commands::review::approve_card,
            commands::review::create_pr,
            commands::review::get_review_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
