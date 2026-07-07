use tauri::{AppHandle, State};
use crate::domains::auth::commands::DbState;
use super::services::run_training;
use super::types::TrainingSummary;

// Training 뷰나 모듈의 State(어댑터 저장 경로 등)
pub struct TrainingState(pub std::sync::Mutex<std::path::PathBuf>);

#[tauri::command]
pub async fn train_lora(
    persona_id: String,
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
) -> Result<TrainingSummary, String> {
    // 백그라운드 태스크로 구동하더라도, 일단은 await로 결과를 반환
    run_training(persona_id, app_handle, db_state.inner()).await
}
