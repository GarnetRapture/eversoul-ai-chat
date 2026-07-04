use super::services::PersonaService;
use super::types::{BondRankingEntry, PersonaConfig, PersonaError, UpdatePersonaRequest};
use crate::domains::auth::commands::DbState;
use crate::domains::settings::commands::SettingsState;
use crate::infrastructure::compress::PersonaLoader;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn persona_list(db_state: State<'_, DbState>) -> Result<Vec<PersonaConfig>, PersonaError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let service = PersonaService::new(&conn);
    service.get_available_personas()
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_update(
    db_state: State<'_, DbState>,
    id: String,
    system_prompt: String,
    greeting: String,
) -> Result<(), PersonaError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let service = PersonaService::new(&conn);
    let req = UpdatePersonaRequest {
        id,
        system_prompt,
        greeting,
    };
    service.update_persona_settings(req)
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_list_archive() -> Result<Vec<String>, PersonaError> {
    Ok(PersonaLoader::list_personas())
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_get_pack(name_en: String) -> Result<serde_json::Value, PersonaError> {
    PersonaLoader::load_persona(&name_en).map_err(|e| PersonaError::Archive(e))
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_select_preset(
    db_state: State<'_, DbState>,
    name_en: String,
) -> Result<PersonaConfig, PersonaError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let service = PersonaService::new(&conn);
    service.load_and_save_preset(&name_en)
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_get_default(
    settings_state: State<'_, SettingsState>,
) -> Result<Option<String>, PersonaError> {
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| PersonaError::Unknown(e.to_string()))?;
    Ok(settings.get_default_persona_id())
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_set_default(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    id: String,
) -> Result<String, PersonaError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| PersonaError::Unknown(e.to_string()))?;
    let service = PersonaService::new(&conn);
    service.set_default_persona_id(&settings, &id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_bond_ranking(
    db_state: State<'_, DbState>,
) -> Result<Vec<BondRankingEntry>, PersonaError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let service = PersonaService::new(&conn);
    service.get_bond_ranking()
}
