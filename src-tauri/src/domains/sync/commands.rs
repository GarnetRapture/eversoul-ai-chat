use super::services::SyncService;
use super::types::{SyncError, SyncResult};
use crate::domains::auth::commands::{DbState, HttpState};
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub async fn sync_run(
    db_state: State<'_, DbState>,
    http_state: State<'_, HttpState>,
) -> Result<SyncResult, SyncError> {
    let service = SyncService::new(&http_state.0);

    // 1. 원격 데이터팩 다운로드 (DB 락 없이 진행되는 비동기 구간)
    let pack = service.fetch_remote_pack().await?;

    // 2. 다운로드 완료 후 짧게 DB 락을 잡아 동기적으로 적재
    let conn = db_state
        .0
        .lock()
        .map_err(|e| SyncError::Database(e.to_string()))?;
    SyncService::persist_pack(&conn, &pack)
}
