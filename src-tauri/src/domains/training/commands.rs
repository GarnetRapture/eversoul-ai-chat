use super::services::TrainingService;
use super::types::{TrainingError, TrainingSummary};
use crate::domains::auth::commands::DbState;
use crate::infrastructure::training::train_persona_lora;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct TrainingState(pub Mutex<PathBuf>);

#[tauri::command(rename_all = "snake_case")]
pub fn training_run(
    db_state: State<'_, DbState>,
    training_state: State<'_, TrainingState>,
    persona_id: String,
) -> Result<TrainingSummary, TrainingError> {

    let examples = {
        let conn = db_state
            .0
            .lock()
            .map_err(|e| TrainingError::Database(e.to_string()))?;
        TrainingService::collect_training_examples(&conn, &persona_id)?
    };
    let examples_used = examples.len();

    let output_path = {
        let adapters_dir = training_state
            .0
            .lock()
            .map_err(|e| TrainingError::Database(e.to_string()))?;
        adapters_dir.join(format!("{persona_id}.safetensors"))
    };

    let report = train_persona_lora(examples, &output_path)
        .map_err(|e| TrainingError::TrainingFailed(e.to_string()))?;

    Ok(TrainingSummary {
        persona_id,
        examples_used,
        steps: report.steps,
        final_loss: report.final_loss,
        adapter_path: report.adapter_path.to_string_lossy().to_string(),
    })
}
