use tauri::{AppHandle, State};
use crate::domains::auth::commands::DbState;
use crate::domains::settings::commands::SettingsState;
use super::services::run_training;
use super::types::{TrainingError, TrainingSummary};

// Training 뷰나 모듈의 State(어댑터 저장 경로 등)
pub struct TrainingState(pub std::sync::Mutex<std::path::PathBuf>);

#[tauri::command]
pub async fn train_lora(
    persona_id: String,
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    training_state: State<'_, TrainingState>,
) -> Result<TrainingSummary, TrainingError> {
    let (language, active_model) = {
        let settings_mgr = settings_state
            .0
            .lock()
            .map_err(|_| TrainingError::state_lock("ko"))?;
        (settings_mgr.get_language(), settings_mgr.get_active_model())
    };
    let adapters_dir = training_state
        .0
        .lock()
        .map_err(|_| TrainingError::state_lock(&language))?
        .clone();
    run_training(
        persona_id,
        app_handle,
        db_state.inner(),
        &adapters_dir,
        &active_model,
        &language,
    )
    .await
}
