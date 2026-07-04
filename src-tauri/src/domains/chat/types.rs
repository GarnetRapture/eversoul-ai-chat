use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub room_id: String,
    pub role: String, // "user" 또는 "assistant" 또는 "system"
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub room_id: String,
    pub content: String,
    pub persona_id: String, // 대화 상대 페르소나 매칭용
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub title: String,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum ChatError {
    #[error("데이터베이스 오류: {0}")]
    Database(String),
    #[error("로컬 LLM 엔진이 실행되지 않았습니다.")]
    LlmEngineNotLoaded,
    #[error("로컬 LLM 추론에 실패했습니다: {0}")]
    LlmInferenceFailed(String),
    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}
