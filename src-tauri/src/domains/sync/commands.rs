use super::services::SyncService;
use super::types::{LocalStatusSnapshot, SyncError, SyncResult};
use crate::domains::auth::commands::{DbState, HttpState};
use crate::startup_debug_log;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub async fn sync_run(
    db_state: State<'_, DbState>,
    http_state: State<'_, HttpState>,
) -> Result<SyncResult, SyncError> {
    startup_debug_log("command:sync_run:start");
    let service = SyncService::new(&http_state.inner().0);

    let pack = match service.fetch_remote_pack().await {
        Ok(pack) => pack,
        Err(err) => {
            if let Ok(conn) = db_state.inner().0.lock() {
                let _ = SyncService::record_failure(&conn, &err.to_string());
            }
            startup_debug_log("command:sync_run:remote_error");
            return Err(err);
        }
    };
    startup_debug_log("command:sync_run:remote_pack_fetched");

    let conn = db_state
        .inner()
        .0
        .lock()
        .map_err(|e| SyncError::Database(e.to_string()))?;
    match SyncService::persist_pack(&conn, &pack) {
        Ok(result) => {
            startup_debug_log("command:sync_run:done");
            Ok(result)
        }
        Err(err) => {
            let _ = SyncService::record_failure(&conn, &err.to_string());
            startup_debug_log("command:sync_run:persist_error");
            Err(err)
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn sync_get_local_status(
    db_state: State<'_, DbState>,
) -> Result<LocalStatusSnapshot, SyncError> {
    startup_debug_log("command:sync_get_local_status:start");
    let conn = db_state
        .inner()
        .0
        .lock()
        .map_err(|e| SyncError::Database(e.to_string()))?;
    let result = SyncService::get_local_status(&conn);
    startup_debug_log("command:sync_get_local_status:done");
    result
}
