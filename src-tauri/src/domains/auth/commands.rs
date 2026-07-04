use tauri::State;
use rusqlite::Connection;
use std::sync::Mutex;
use crate::infrastructure::http::HttpManager;
use super::types::{LoginRequest, UserSession, AuthError};
use super::services::AuthService;

pub struct DbState(pub Mutex<Connection>);
pub struct HttpState(pub HttpManager);

#[tauri::command]
pub async fn auth_login(
    db_state: State<'_, DbState>,
    http_state: State<'_, HttpState>,
    email: String,
    token: String,
) -> Result<UserSession, AuthError> {
    let conn = db_state.0.lock().map_err(|e| AuthError::Database(e.to_string()))?;
    let req = LoginRequest { email, token };
    let service = AuthService::new(&conn, &http_state.0);
    
    service.login(req).await
}

#[tauri::command]
pub fn auth_logout(
    db_state: State<'_, DbState>,
    http_state: State<'_, HttpState>,
) -> Result<(), AuthError> {
    let conn = db_state.0.lock().map_err(|e| AuthError::Database(e.to_string()))?;
    let service = AuthService::new(&conn, &http_state.0);
    service.logout()
}

#[tauri::command]
pub fn auth_get_session(
    db_state: State<'_, DbState>,
    http_state: State<'_, HttpState>,
) -> Result<Option<UserSession>, AuthError> {
    let conn = db_state.0.lock().map_err(|e| AuthError::Database(e.to_string()))?;
    let service = AuthService::new(&conn, &http_state.0);
    service.current_session()
}
