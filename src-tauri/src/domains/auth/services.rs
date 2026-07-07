use super::repositories::AuthRepository;
use super::types::{AuthError, LoginRequest, UserSession};
use rusqlite::Connection;

pub struct AuthService;

impl AuthService {
    pub fn local_auth_session(conn: &Connection, req: LoginRequest) -> Result<UserSession, AuthError> {
        let username = req.email.split('@').next().unwrap_or("LocalUser").to_string();
        let session = UserSession {
            token: req.token,
            email: req.email,
            username,
            created_at: String::new(),
        };

        Self::persist_session(conn, &session)?;
        Ok(session)
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
