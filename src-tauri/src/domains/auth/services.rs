use super::repositories::AuthRepository;
use super::types::{AuthError, LoginRequest, UserSession};
use crate::infrastructure::http::HttpManager;
use rusqlite::Connection;

pub struct AuthService<'a> {
    http: &'a HttpManager,
}

impl<'a> AuthService<'a> {
    pub fn new(http: &'a HttpManager) -> Self {
        Self { http }
    }

    pub async fn verify_remote_session(&self, req: LoginRequest) -> Result<UserSession, AuthError> {
        let path = format!("/auth/verify?email={}&token={}", req.email, req.token);

        self.http
            .get::<UserSession>(&path)
            .await
            .map_err(|e| AuthError::Network(e.to_string()))
    }

    pub fn persist_session(conn: &Connection, session: &UserSession) -> Result<(), AuthError> {
        AuthRepository::save_session(conn, session).map_err(|e| AuthError::Database(e.to_string()))
    }

    pub fn current_session(conn: &Connection) -> Result<Option<UserSession>, AuthError> {
        AuthRepository::get_session(conn).map_err(|e| AuthError::Database(e.to_string()))
    }

    pub fn logout(conn: &Connection) -> Result<(), AuthError> {
        AuthRepository::clear_session(conn).map_err(|e| AuthError::Database(e.to_string()))
    }
}
