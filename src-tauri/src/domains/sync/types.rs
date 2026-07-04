use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub synced_items: usize,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDataPack {
    pub personas: Vec<crate::domains::persona::types::PersonaConfig>,
    pub knowledges: Vec<crate::domains::knowledge::types::KnowledgePayload>,
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

