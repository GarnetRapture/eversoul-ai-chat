use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub token: String,
    pub email: String,
    pub username: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub session: Option<UserSession>,
    pub error_message: Option<String>,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum AuthError {
    #[error("데이터베이스 오류: {0}")]
    Database(String),
    #[error("네트워크 통신 오류: {0}")]
    Network(String),
    #[error("유효하지 않은 계정 정보 또는 만료된 인증 토큰입니다.")]
    InvalidCredentials,
    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}
