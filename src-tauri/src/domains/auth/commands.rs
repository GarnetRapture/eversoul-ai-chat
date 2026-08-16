use super::services::AuthService;
use super::types::{AuthError, LoginRequest, UserSession};
use crate::domains::settings::commands::SettingsState;
use crate::infrastructure::database::DbPool;
use tauri::State;

pub struct DbState(pub DbPool);

fn command_language(settings_state: &State<'_, SettingsState>) -> String {
    settings_state
        .inner()
        .0
        .lock()
        .map(|settings| settings.get_language())
        .unwrap_or_else(|_| "ko".to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn auth_login(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    email: String,
    token: String,
) -> Result<UserSession, AuthError> {
    let language = command_language(&settings_state);
    let req = LoginRequest { email, token };

    let conn = db_state
        .0
        .get()
        .map_err(|e| AuthError::database(&language, &e.to_string()))?;

    let session = AuthService::local_auth_session(&conn, req, &language)?;

    Ok(session)
}

#[tauri::command(rename_all = "snake_case")]
pub fn auth_logout(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
) -> Result<(), AuthError> {
    let language = command_language(&settings_state);
    let conn = db_state
        .0
        .get()
        .map_err(|e| AuthError::database(&language, &e.to_string()))?;
    AuthService::logout(&conn, &language)
}

#[tauri::command(rename_all = "snake_case")]
pub fn auth_get_session(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
) -> Result<Option<UserSession>, AuthError> {
    let language = command_language(&settings_state);
    let conn = db_state
        .0
        .get()
        .map_err(|e| AuthError::database(&language, &e.to_string()))?;
    AuthService::current_session(&conn, &language)
}
