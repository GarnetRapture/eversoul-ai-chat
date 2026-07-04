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

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum StyleError {
    #[error("데이터베이스 오류: {0}")]
    Database(String),
    #[error("스타일 프로필을 찾을 수 없습니다: {0}")]
    NotFound(String),
    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}
