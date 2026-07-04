pub mod domains;
pub mod infrastructure;

use std::sync::Mutex;
use tauri::Manager;

use crate::domains::auth::commands::{DbState, HttpState};
use crate::domains::llm::commands::LlmState;
use crate::infrastructure::database::DatabaseManager;
use crate::infrastructure::http::HttpManager;

use crate::domains::auth::commands::{auth_get_session, auth_login, auth_logout};
use crate::domains::chat::commands::{
    chat_create_room, chat_create_session_room, chat_get_latest_session_room, chat_list_messages,
    chat_list_rooms, chat_send_message,
};
use crate::domains::knowledge::commands::knowledge_search;
use crate::domains::llm::commands::{llm_infer, llm_load, llm_status};
use crate::domains::persona::commands::{
    persona_bond_ranking, persona_get_default, persona_get_pack, persona_list,
    persona_list_archive, persona_select_preset, persona_set_default, persona_update,
};
use crate::domains::settings::commands::{settings_get, settings_reset, SettingsState};
use crate::domains::style::commands::{
    style_get_active, style_list, style_select_active, style_update,
};
use crate::domains::sync::commands::sync_run;
use crate::domains::training::commands::{training_run, TrainingState};
use crate::infrastructure::settings::SettingsManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {

            let db_path = app
                .path()
                .app_local_data_dir()
                .map(|p| {
                    let db_dir = p.join("database");
                    let _ = std::fs::create_dir_all(&db_dir);
                    db_dir.join("eversoul.db")
                })
                .unwrap_or_else(|_| {
                    let db_dir = std::path::PathBuf::from("database");
                    let _ = std::fs::create_dir_all(&db_dir);
                    db_dir.join("eversoul.db")
                });

            let db_mgr = DatabaseManager::new(db_path);
            let conn = db_mgr
                .connect()
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

            let settings_path = app
                .path()
                .app_local_data_dir()
                .map(|p| {
                    let config_dir = p.join("config");
                    let _ = std::fs::create_dir_all(&config_dir);
                    config_dir.join("settings.ini")
                })
                .unwrap_or_else(|_| {
                    let config_dir = std::path::PathBuf::from("config");
                    let _ = std::fs::create_dir_all(&config_dir);
                    config_dir.join("settings.ini")
                });
            let settings_mgr = SettingsManager::new(settings_path);

            let adapters_dir = app
                .path()
                .app_local_data_dir()
                .map(|p| p.join("lora_adapters"))
                .unwrap_or_else(|_| std::path::PathBuf::from("lora_adapters"));
            let _ = std::fs::create_dir_all(&adapters_dir);

            let check_empty: Result<i64, _> =
                conn.query_row("SELECT COUNT(*) FROM persona_profile", [], |r| r.get(0));
            if let Ok(count) = check_empty {
                if count == 0 {
                    let service = crate::domains::persona::services::PersonaService::new(&conn);
                    let archive_names =
                        crate::infrastructure::compress::PersonaLoader::list_personas();
                    for name in archive_names {
                        let _ = service.load_and_save_preset(&name);
                    }
                }
            }

            let http_mgr = HttpManager::new("https://api.eversoul-ai.chat".to_string());

            app.manage(DbState(Mutex::new(conn)));
            app.manage(HttpState(http_mgr));
            app.manage(LlmState(Mutex::new(None)));
            app.manage(SettingsState(Mutex::new(settings_mgr)));
            app.manage(TrainingState(Mutex::new(adapters_dir)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_login,
            auth_logout,
            auth_get_session,
            sync_run,
            persona_list,
            persona_update,
            persona_list_archive,
            persona_get_pack,
            persona_select_preset,
            persona_get_default,
            persona_set_default,
            persona_bond_ranking,
            knowledge_search,
            chat_create_room,
            chat_create_session_room,
            chat_get_latest_session_room,
            chat_list_rooms,
            chat_list_messages,
            chat_send_message,
            llm_load,
            llm_status,
            llm_infer,
            style_list,
            style_update,
            style_select_active,
            style_get_active,
            settings_get,
            settings_reset,
            training_run
        ])
        .run(tauri::generate_context!())
        .expect("error while running eversoul-ai-chat application");
}
