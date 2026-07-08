use super::services::ModuleService;
use super::types::{ImportedModule, ModuleControl, ModuleError};
use crate::domains::settings::commands::SettingsState;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn modules_list(
    settings_state: State<'_, SettingsState>,
) -> Result<Vec<ImportedModule>, ModuleError> {
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| ModuleError::Storage(e.to_string()))?;
    ModuleService::list(&settings)
}

#[tauri::command(rename_all = "snake_case")]
pub fn modules_import_from_path(
    settings_state: State<'_, SettingsState>,
    path: String,
) -> Result<ImportedModule, ModuleError> {
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| ModuleError::Storage(e.to_string()))?;
    ModuleService::import_from_path(&settings, &path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn modules_set_enabled(
    settings_state: State<'_, SettingsState>,
    id: String,
    enabled: bool,
) -> Result<Vec<ImportedModule>, ModuleError> {
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| ModuleError::Storage(e.to_string()))?;
    ModuleService::set_enabled(&settings, &id, enabled)
}

#[tauri::command(rename_all = "snake_case")]
pub fn modules_delete(
    settings_state: State<'_, SettingsState>,
    id: String,
) -> Result<Vec<ImportedModule>, ModuleError> {
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| ModuleError::Storage(e.to_string()))?;
    ModuleService::delete(&settings, &id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn modules_update_controls(
    settings_state: State<'_, SettingsState>,
    id: String,
    controls: Vec<ModuleControl>,
) -> Result<Vec<ImportedModule>, ModuleError> {
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| ModuleError::Storage(e.to_string()))?;
    ModuleService::update_controls(&settings, &id, controls)
}
