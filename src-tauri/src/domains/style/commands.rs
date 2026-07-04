use super::services::StyleService;
use super::types::{StyleError, StyleProfile, UpdateStyleRequest};
use crate::domains::auth::commands::DbState;
use crate::domains::settings::commands::SettingsState;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn style_list(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
) -> Result<Vec<StyleProfile>, StyleError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| StyleError::Database(e.to_string()))?;
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| StyleError::Unknown(e.to_string()))?;
    let service = StyleService::new(&conn);
    service.get_available_styles(settings.get_active_style_id().as_deref())
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
    settings_state: State<'_, SettingsState>,
    id: String,
) -> Result<StyleProfile, StyleError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| StyleError::Database(e.to_string()))?;
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| StyleError::Unknown(e.to_string()))?;
    let service = StyleService::new(&conn);
    let mut style = service.get_style_for_activation(&id)?;

    settings
        .set_active_style_id(&id)
        .map_err(|e| StyleError::Unknown(e.to_string()))?;
    style.is_active = true;
    Ok(style)
}

#[tauri::command(rename_all = "snake_case")]
pub fn style_get_active(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
) -> Result<Option<StyleProfile>, StyleError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| StyleError::Database(e.to_string()))?;
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| StyleError::Unknown(e.to_string()))?;

    match settings.get_active_style_id() {
        Some(id) => {
            let service = StyleService::new(&conn);
            let style = service.get_style_by_id(&id)?;
            Ok(style.map(|mut s| {
                s.is_active = true;
                s
            }))
        }
        None => Ok(None),
    }
}
