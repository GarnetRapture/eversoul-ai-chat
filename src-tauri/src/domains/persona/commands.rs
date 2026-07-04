use super::services::PersonaService;
use super::types::{PersonaConfig, PersonaError, UpdatePersonaRequest};
use crate::domains::auth::commands::DbState;
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

/// 아카이브에 포함된 모든 정령의 영어명 리스트 조회
#[tauri::command(rename_all = "snake_case")]
pub fn persona_list_archive() -> Result<Vec<String>, PersonaError> {
    Ok(PersonaLoader::list_personas())
}

/// 특정 정령의 영어명을 기준으로 완벽압축 바이너리(personas.bin)에서 온디맨드 부분 해제하여 획득
#[tauri::command(rename_all = "snake_case")]
pub fn persona_get_pack(name_en: String) -> Result<serde_json::Value, PersonaError> {
    PersonaLoader::load_persona(&name_en).map_err(|e| PersonaError::Archive(e))
}

/// 특정 정령의 영어명을 프리셋으로 선택해 완벽압축 팩을 해제하고 로컬 DB에 활성 주입
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
