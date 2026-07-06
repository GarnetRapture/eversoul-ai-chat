use super::services::PersonaService;
use super::types::{
    BondRankingEntry, FamiliarityEntry, PersonaConfig, PersonaError, UpdatePersonaRequest,
};
use crate::domains::auth::commands::DbState;
use crate::domains::settings::commands::SettingsState;
use crate::infrastructure::compress::PersonaLoader;
use crate::startup_debug_log;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn persona_list(db_state: State<'_, DbState>) -> Result<Vec<PersonaConfig>, PersonaError> {
    startup_debug_log("command:persona_list:start");
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let service = PersonaService::new(&conn);
    let result = service.get_available_personas();
    startup_debug_log("command:persona_list:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_update(
    db_state: State<'_, DbState>,
    id: String,
    system_prompt: String,
    greeting: String,
) -> Result<(), PersonaError> {
    startup_debug_log("command:persona_update:start");
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
    let result = service.update_persona_settings(req);
    startup_debug_log("command:persona_update:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_list_archive() -> Result<Vec<String>, PersonaError> {
    startup_debug_log("command:persona_list_archive:start");
    let result = PersonaLoader::list_personas();
    startup_debug_log("command:persona_list_archive:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_get_pack(name_en: String) -> Result<serde_json::Value, PersonaError> {
    startup_debug_log("command:persona_get_pack:start");
    let result = PersonaLoader::load_persona(&name_en).map_err(|e| PersonaError::Archive(e));
    startup_debug_log("command:persona_get_pack:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_select_preset(
    db_state: State<'_, DbState>,
    name_en: String,
) -> Result<PersonaConfig, PersonaError> {
    startup_debug_log("command:persona_select_preset:start");
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let service = PersonaService::new(&conn);
    let result = service.load_and_save_preset(&name_en);
    startup_debug_log("command:persona_select_preset:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_get_default(
    settings_state: State<'_, SettingsState>,
) -> Result<Option<String>, PersonaError> {
    startup_debug_log("command:persona_get_default:start");
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| PersonaError::Unknown(e.to_string()))?;
    let result = settings.get_default_persona_id();
    startup_debug_log("command:persona_get_default:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_set_default(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    id: String,
) -> Result<String, PersonaError> {
    startup_debug_log("command:persona_set_default:start");
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| PersonaError::Unknown(e.to_string()))?;
    let service = PersonaService::new(&conn);
    let result = service.set_default_persona_id(&settings, &id);
    startup_debug_log("command:persona_set_default:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_bond_ranking(
    db_state: State<'_, DbState>,
) -> Result<Vec<BondRankingEntry>, PersonaError> {
    startup_debug_log("command:persona_bond_ranking:start");
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let service = PersonaService::new(&conn);
    let result = service.get_bond_ranking();
    startup_debug_log("command:persona_bond_ranking:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn persona_familiarity_list(
    db_state: State<'_, DbState>,
) -> Result<Vec<FamiliarityEntry>, PersonaError> {
    startup_debug_log("command:persona_familiarity_list:start");
    let conn = db_state
        .0
        .lock()
        .map_err(|e| PersonaError::Database(e.to_string()))?;
    let service = PersonaService::new(&conn);
    let result = service.get_familiarity_list();
    startup_debug_log("command:persona_familiarity_list:done");
    result
}
