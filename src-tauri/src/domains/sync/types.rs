use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub synced_items: usize,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStatusSnapshot {
    pub persona_count: usize,
    pub chat_room_count: usize,
    pub chat_message_count: usize,
    pub style_count: usize,
    pub knowledge_chunk_count: usize,
    pub memory_count: usize,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDataPack {
    pub personas: Vec<crate::domains::persona::types::PersonaConfig>,
    pub knowledges: Vec<crate::domains::knowledge::types::KnowledgePayload>,
    pub styles: Vec<crate::domains::style::types::StyleProfile>,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum SyncError {
    #[error("데이터베이스 오류: {0}")]
    Database(String),
    #[error("동기화 서버 네트워크 통신 오류: {0}")]
    Network(String),
    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}
