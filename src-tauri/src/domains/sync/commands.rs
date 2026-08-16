use super::services::SyncService;
use super::types::{LocalStatusSnapshot, SyncError, SyncResult};
use crate::domains::auth::commands::DbState;
use crate::domains::settings::commands::SettingsState;
use crate::startup_debug_log;
use tauri::State;

fn command_language(settings_state: &State<'_, SettingsState>) -> String {
    settings_state
        .inner()
        .0
        .lock()
        .map(|settings| settings.get_language())
        .unwrap_or_else(|_| "ko".to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sync_run(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
) -> Result<SyncResult, SyncError> {
    startup_debug_log("command:sync_run:start");
    let language = command_language(&settings_state);

    let pack = match SyncService::extract_local_pack() {
        Ok(pack) => pack,
        Err(err) => {
            if let Ok(conn) = db_state.inner().0.get() {
                let _ = SyncService::record_failure(&conn, &err.to_string(), &language);
            }
            startup_debug_log("command:sync_run:local_extract_error");
            return Err(err);
        }
    };
    startup_debug_log("command:sync_run:local_pack_extracted");

    let conn = db_state
        .inner()
        .0
        .get()
        .map_err(|e| SyncError::database(&language, &e.to_string()))?;
    match SyncService::persist_pack(&conn, &pack, &language) {
        Ok(result) => {
            startup_debug_log("command:sync_run:done");
            Ok(result)
        }
        Err(err) => {
            let _ = SyncService::record_failure(&conn, &err.to_string(), &language);
            startup_debug_log("command:sync_run:persist_error");
            Err(err)
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn sync_get_local_status(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
) -> Result<LocalStatusSnapshot, SyncError> {
    startup_debug_log("command:sync_get_local_status:start");
    let language = command_language(&settings_state);
    let conn = db_state
        .inner()
        .0
        .get()
        .map_err(|e| SyncError::database(&language, &e.to_string()))?;
    let result = SyncService::get_local_status(&conn, &language);
    startup_debug_log("command:sync_get_local_status:done");
    result
}
