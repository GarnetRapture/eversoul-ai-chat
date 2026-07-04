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

    /// 원격 API 서버를 통해 토큰을 검증하여 세션을 획득한다.
    ///
    /// DB 커넥션을 들고 있지 않은 순수 비동기 구간으로 분리하여,
    /// 이 `.await` 동안 SQLite `MutexGuard`(non-Send)가 스레드 경계를 넘지 않도록 한다.
    pub async fn verify_remote_session(&self, req: LoginRequest) -> Result<UserSession, AuthError> {
        let path = format!("/auth/verify?email={}&token={}", req.email, req.token);

        // 원격 API 서버로 실제 인증 요청 (더미 폴백 없음)
        self.http
            .get::<UserSession>(&path)
            .await
            .map_err(|e| AuthError::Network(e.to_string()))
    }

    /// 검증이 끝난 세션을 로컬 SQLite 3 데이터베이스에 영속화한다 (동기).
    pub fn persist_session(conn: &Connection, session: &UserSession) -> Result<(), AuthError> {
        AuthRepository::save_session(conn, session).map_err(|e| AuthError::Database(e.to_string()))
    }

    /// 현재 데이터베이스에 기록된 유효한 세션을 확인한다.
    pub fn current_session(conn: &Connection) -> Result<Option<UserSession>, AuthError> {
        AuthRepository::get_session(conn).map_err(|e| AuthError::Database(e.to_string()))
    }

    /// 세션을 삭제하여 로그아웃 처리를 수행한다.
    pub fn logout(conn: &Connection) -> Result<(), AuthError> {
        AuthRepository::clear_session(conn).map_err(|e| AuthError::Database(e.to_string()))
    }
}
