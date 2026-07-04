use super::services::AuthService;
use super::types::{AuthError, LoginRequest, UserSession};
use crate::infrastructure::http::HttpManager;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub struct DbState(pub Mutex<Connection>);
pub struct HttpState(pub HttpManager);

#[tauri::command(rename_all = "snake_case")]
pub async fn auth_login(
    db_state: State<'_, DbState>,
    http_state: State<'_, HttpState>,
    email: String,
    token: String,
) -> Result<UserSession, AuthError> {
    let req = LoginRequest { email, token };
    let service = AuthService::new(&http_state.0);

    let session = service.verify_remote_session(req).await?;

    let conn = db_state
        .0
        .lock()
        .map_err(|e| AuthError::Database(e.to_string()))?;
    AuthService::persist_session(&conn, &session)?;

    Ok(session)
}

#[tauri::command(rename_all = "snake_case")]
pub fn auth_logout(db_state: State<'_, DbState>) -> Result<(), AuthError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| AuthError::Database(e.to_string()))?;
    AuthService::logout(&conn)
}

#[tauri::command(rename_all = "snake_case")]
pub fn auth_get_session(db_state: State<'_, DbState>) -> Result<Option<UserSession>, AuthError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| AuthError::Database(e.to_string()))?;
    AuthService::current_session(&conn)
}
