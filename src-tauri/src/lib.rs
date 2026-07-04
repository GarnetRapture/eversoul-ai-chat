pub mod domains;
pub mod infrastructure;

use std::sync::Mutex;
use tauri::Manager;

use crate::domains::auth::commands::{DbState, HttpState};
use crate::domains::llm::commands::LlmState;
use crate::infrastructure::database::DatabaseManager;
use crate::infrastructure::http::HttpManager;

// Tauri Commands 일괄 임포트
use crate::domains::auth::commands::{auth_get_session, auth_login, auth_logout};
use crate::domains::chat::commands::{
    chat_create_room, chat_list_messages, chat_list_rooms, chat_send_message,
};
use crate::domains::knowledge::commands::knowledge_search;
use crate::domains::llm::commands::{llm_infer, llm_load, llm_status};
use crate::domains::persona::commands::{
    persona_get_pack, persona_list, persona_list_archive, persona_select_preset, persona_update,
};
use crate::domains::style::commands::{
    style_get_active, style_list, style_select_active, style_update,
};
use crate::domains::sync::commands::sync_run;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 1. SQLite3 데이터베이스 파일 경로 지정 및 초기화
            // 앱 로컬 데이터 경로 획득 불가능 시 현재 디렉토리에 생성하여 SQLite 3 연동 보장
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

            // SQLite3 DB: 페르소나 데이터팩 초기 자동 마이그레이션
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

            // 2. HTTP Manager 초기 설정
            let http_mgr = HttpManager::new("https://api.eversoul-ai.chat".to_string());

            // 3. Tauri 전역 리소스(State) 등록
            app.manage(DbState(Mutex::new(conn)));
            app.manage(HttpState(http_mgr));
            app.manage(LlmState(Mutex::new(None)));

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
            knowledge_search,
            chat_create_room,
            chat_list_rooms,
            chat_list_messages,
            chat_send_message,
            llm_load,
            llm_status,
            llm_infer,
            style_list,
            style_update,
            style_select_active,
            style_get_active
        ])
        .run(tauri::generate_context!())
        .expect("error while running eversoul-ai-chat application");
}
