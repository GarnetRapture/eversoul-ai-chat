pub mod domains;
pub mod infrastructure;

use std::sync::Mutex;
use tauri::Manager;

use crate::domains::auth::commands::DbState;
use crate::domains::llm::commands::LlmState;
use crate::infrastructure::database::DatabaseManager;

use crate::domains::auth::commands::{auth_get_session, auth_login, auth_logout};
use crate::domains::chat::commands::{
    chat_create_room, chat_create_session_room, chat_get_evertalk_session_room,
    chat_get_latest_session_room, chat_list_messages, chat_list_messages_for_persona,
    chat_list_rooms, chat_prepare_persona_cache, chat_send_message,
};
use crate::domains::knowledge::commands::knowledge_search;
use crate::domains::llm::commands::{
    llm_active_sessions, llm_cancel_request, llm_download_model, llm_infer, llm_infer_stream,
    llm_load, llm_request_statuses, llm_self_test, llm_session_statuses,
    llm_status, llm_unload, llm_verify_model,
};
use crate::domains::persona::commands::{
    persona_bond_ranking, persona_familiarity_list, persona_get_default, persona_get_pack,
    persona_list, persona_list_archive, persona_select_preset, persona_set_default, persona_update,
};
use crate::domains::settings::commands::{
    settings_complete_initial_setup, settings_detect_hardware, settings_get, settings_reset,
    settings_set_language, settings_set_performance_tier, settings_set_setup_stage, settings_set_show_reasoning, SettingsState,
};
use crate::domains::style::commands::{
    style_get_active, style_list, style_select_active, style_update,
};
use crate::domains::sync::commands::{sync_get_local_status, sync_run};
use crate::domains::training::commands::{train_lora, TrainingState};
// [TTS 연동 보류] voice_synthesize: 합성 음성 품질 미흡으로 TTS 연동 보류 (2026-07-07). 재개 시 목록에 추가.
use crate::domains::voice::commands::{voice_get, voice_list};
use crate::infrastructure::settings::SettingsManager;

pub(crate) fn startup_debug_log(stage: &str) {
    use std::io::Write;

    eprintln!("[eversoul-startup] {stage}");
    let _ = std::io::stderr().flush();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    startup_debug_log("run:start");

    let app = tauri::Builder::default()
        .plugin({
            startup_debug_log("plugin:opener:init:before");
            let plugin = tauri_plugin_opener::init();
            startup_debug_log("plugin:opener:init:after");
            plugin
        })
        .plugin({
            startup_debug_log("plugin:shell:init:before");
            let plugin = tauri_plugin_shell::init();
            startup_debug_log("plugin:shell:init:after");
            plugin
        })
        .plugin({
            startup_debug_log("plugin:dialog:init:before");
            let plugin = tauri_plugin_dialog::init();
            startup_debug_log("plugin:dialog:init:after");
            plugin
        })
        .plugin({
            startup_debug_log("plugin:fs:init:before");
            let plugin = tauri_plugin_fs::init();
            startup_debug_log("plugin:fs:init:after");
            plugin
        })
        .setup(|app| {
            startup_debug_log("setup:start");
            
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));

            let db_path = {
                let db_dir = exe_dir.join("database");
                let _ = std::fs::create_dir_all(&db_dir);
                db_dir.join("eversoul.db")
            };

            startup_debug_log("setup:database:path_ready");
            let db_mgr = DatabaseManager::new(db_path);
            let conn = db_mgr
                .connect()
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
            startup_debug_log("setup:database:connected");

            let settings_path = {
                let config_dir = exe_dir.join("config");
                let _ = std::fs::create_dir_all(&config_dir);
                config_dir.join("settings.ini")
            };
            let settings_mgr = SettingsManager::new(settings_path);
            let _ = settings_mgr.ensure_initialized();
            startup_debug_log("setup:settings:ready");

            let adapters_dir = {
                let p = exe_dir.join("lora_adapters");
                let _ = std::fs::create_dir_all(&p);
                p
            };
            startup_debug_log("setup:lora_adapters:ready");

            app.manage(DbState(Mutex::new(conn)));
            startup_debug_log("setup:state:db");
            app.manage(LlmState(Mutex::new(None)));
            startup_debug_log("setup:state:llm");
            app.manage(SettingsState(Mutex::new(settings_mgr)));
            startup_debug_log("setup:state:settings");
            app.manage(TrainingState(Mutex::new(adapters_dir)));
            startup_debug_log("setup:state:training");

            startup_debug_log("setup:done");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_login,
            auth_logout,
            auth_get_session,
            sync_run,
            sync_get_local_status,
            persona_list,
            persona_update,
            persona_list_archive,
            persona_get_pack,
            persona_select_preset,
            persona_get_default,
            persona_set_default,
            persona_bond_ranking,
            persona_familiarity_list,
            knowledge_search,
            chat_create_room,
            chat_create_session_room,
            chat_get_evertalk_session_room,
            chat_get_latest_session_room,
            chat_list_rooms,
            chat_list_messages,
            chat_list_messages_for_persona,
            chat_prepare_persona_cache,
            chat_send_message,
            llm_load,
            llm_download_model,
            llm_unload,
            llm_status,
            llm_infer,
            llm_infer_stream,
            llm_cancel_request,
            llm_active_sessions,
            llm_session_statuses,
            llm_request_statuses,
            llm_verify_model,
            llm_self_test,
            style_list,
            style_update,
            style_select_active,
            style_get_active,
            settings_get,
            settings_reset,
            settings_set_language,
            settings_set_performance_tier,
            settings_set_setup_stage,
            settings_set_show_reasoning,
            settings_detect_hardware,
            settings_complete_initial_setup,
            train_lora,
            voice_list,
            voice_get // [TTS 연동 보류] voice_synthesize: 합성 음성 품질 미흡으로 TTS 연동 보류 (2026-07-07).
        ])
        .build({
            startup_debug_log("context:generate:before");
            let context = tauri::generate_context!();
            startup_debug_log("context:generate:after");
            startup_debug_log("tauri:build:before");
            context
        })
        .expect("error while building eversoul-ai-chat application");

    startup_debug_log("tauri:build:after");
    startup_debug_log("tauri:run:before");
    app.run(|_app_handle, event| {
        if !matches!(event, tauri::RunEvent::MainEventsCleared) {
            startup_debug_log(&format!("tauri:event:{event:?}"));
        }
    });
    startup_debug_log("tauri:run:returned");
}
