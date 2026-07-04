use super::services::StyleService;
use super::types::{StyleError, StyleProfile, UpdateStyleRequest};
use crate::domains::auth::commands::DbState;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn style_list(db_state: State<'_, DbState>) -> Result<Vec<StyleProfile>, StyleError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| StyleError::Database(e.to_string()))?;
    let service = StyleService::new(&conn);
    service.get_available_styles()
}

#[tauri::command(rename_all = "snake_case")]
pub fn style_update(
    db_state: State<'_, DbState>,
    id: String,
    tone: String,
    formality: String,
    emoji_usage: bool,
    speech_rules: String,
) -> Result<(), StyleError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| StyleError::Database(e.to_string()))?;
    let service = StyleService::new(&conn);
    let req = UpdateStyleRequest {
        id,
        tone,
        formality,
        emoji_usage,
        speech_rules,
    };
    service.update_style_settings(req)
}

#[tauri::command(rename_all = "snake_case")]
pub fn style_select_active(
    db_state: State<'_, DbState>,
    id: String,
) -> Result<StyleProfile, StyleError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| StyleError::Database(e.to_string()))?;
    let service = StyleService::new(&conn);
    service.select_active_style(&id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn style_get_active(db_state: State<'_, DbState>) -> Result<Option<StyleProfile>, StyleError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| StyleError::Database(e.to_string()))?;
    let service = StyleService::new(&conn);
    service.get_active_style()
}
