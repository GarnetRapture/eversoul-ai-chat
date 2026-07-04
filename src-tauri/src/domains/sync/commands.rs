use tauri::State;
use crate::domains::auth::commands::{DbState, HttpState};
use super::types::{SyncResult, SyncError};
use super::services::SyncService;

#[tauri::command]
pub async fn sync_run(
    db_state: State<'_, DbState>,
    http_state: State<'_, HttpState>,
) -> Result<SyncResult, SyncError> {
    let conn = db_state.0.lock().map_err(|e| SyncError::Database(e.to_string()))?;
    let service = SyncService::new(&conn, &http_state.0);
    
    service.run_synchronization().await
}
