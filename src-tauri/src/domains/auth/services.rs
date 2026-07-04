use rusqlite::Connection;
use crate::infrastructure::http::HttpManager;
use super::types::{LoginRequest, UserSession, AuthError};
use super::repositories::AuthRepository;

pub struct AuthService<'a> {
    conn: &'a Connection,
    http: &'a HttpManager,
}

impl<'a> AuthService<'a> {
    pub fn new(conn: &'a Connection, http: &'a HttpManager) -> Self {
        Self { conn, http }
    }

    /// 원격 API 서버를 통해 토큰을 검증하고 세션을 수립한 뒤 로컬 SQLite에 저장한다.
    pub async fn login(&self, req: LoginRequest) -> Result<UserSession, AuthError> {
        let path = format!("/auth/verify?email={}&token={}", req.email, req.token);
        
        // 원격 API 서버로 실제 인증 요청 (더미 폴백 없음)
        let session = self.http.get::<UserSession>(&path).await
            .map_err(|e| AuthError::Network(e.to_string()))?;

        // 로컬 SQLite 3 데이터베이스에 세션 정보 영속화
        AuthRepository::save_session(self.conn, &session)
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(session)
    }

    /// 현재 데이터베이스에 기록된 유효한 세션을 확인한다.
    pub fn current_session(&self) -> Result<Option<UserSession>, AuthError> {
        AuthRepository::get_session(self.conn)
            .map_err(|e| AuthError::Database(e.to_string()))
    }

    /// 세션을 삭제하여 로그아웃 처리를 수행한다.
    pub fn logout(&self) -> Result<(), AuthError> {
        AuthRepository::clear_session(self.conn)
            .map_err(|e| AuthError::Database(e.to_string()))
    }
}
