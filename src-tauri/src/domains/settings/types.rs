use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_persona_id: Option<String>,
    pub active_style_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetSummary {
    pub cleared_chat_rooms: usize,
    pub cleared_chat_messages: usize,
    pub cleared_personas: usize,
    pub cleared_styles: usize,
    pub cleared_knowledge_chunks: usize,
    pub cleared_persona_memories: usize,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum SettingsError {
    #[error("설정 파일 접근 실패: {0}")]
    Io(String),
    #[error("데이터베이스 오류: {0}")]
    Database(String),
}
