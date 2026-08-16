use crate::infrastructure::i18n::pick;
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

/// `code`는 프론트엔드 프로그래밍적 분기용, `message`는 이미 요청 시점 언어
/// (ko/en/zh_cn)로 렌더링된 텍스트다. Tauri IPC로 그대로 직렬화된다.
#[derive(Debug, Clone, Serialize)]
pub struct AuthError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AuthError {}

impl AuthError {
    pub fn database(language: &str, detail: &str) -> Self {
        Self {
            code: "database",
            message: pick(
                language,
                format!("데이터베이스 오류: {detail}"),
                format!("Database error: {detail}"),
                format!("数据库错误：{detail}"),
            ),
        }
    }

    pub fn invalid_credentials(language: &str) -> Self {
        Self {
            code: "invalid_credentials",
            message: pick(
                language,
                "유효하지 않은 계정 정보 또는 만료된 인증 토큰입니다.".to_string(),
                "Invalid account credentials or an expired authentication token.".to_string(),
                "账户信息无效或认证令牌已过期。".to_string(),
            ),
        }
    }
}
