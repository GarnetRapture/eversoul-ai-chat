use super::services::SettingsService;
use super::types::{AppSettings, ResetSummary, SettingsError};
use crate::domains::auth::commands::DbState;
use crate::infrastructure::settings::SettingsManager;
use std::sync::Mutex;
use tauri::State;

pub struct SettingsState(pub Mutex<SettingsManager>);

#[tauri::command(rename_all = "snake_case")]
pub fn settings_get(settings_state: State<'_, SettingsState>) -> Result<AppSettings, SettingsError> {
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    Ok(SettingsService::get_settings(&settings))
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_reset(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
) -> Result<ResetSummary, SettingsError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| SettingsError::Database(e.to_string()))?;
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    SettingsService::reset_all(&conn, &settings)
}
