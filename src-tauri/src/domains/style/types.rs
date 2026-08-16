use crate::infrastructure::i18n::pick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleProfile {
    pub id: String,
    pub name: String,
    pub tone: String,
    pub formality: String,
    pub emoji_usage: bool,
    pub speech_rules: String,
    pub example_phrases: String,
    pub raw_json: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStyleRequest {
    pub id: String,
    pub tone: String,
    pub formality: String,
    pub emoji_usage: bool,
    pub speech_rules: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StyleError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for StyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for StyleError {}

impl StyleError {
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

    pub fn not_found(language: &str, id: &str) -> Self {
        Self {
            code: "not_found",
            message: pick(
                language,
                format!("스타일 프로필을 찾을 수 없습니다: {id}"),
                format!("Style profile not found: {id}"),
                format!("找不到风格档案：{id}"),
            ),
        }
    }

    pub fn unknown(language: &str, detail: &str) -> Self {
        Self {
            code: "unknown",
            message: pick(
                language,
                format!("알 수 없는 오류: {detail}"),
                format!("Unknown error: {detail}"),
                format!("未知错误：{detail}"),
            ),
        }
    }
}
