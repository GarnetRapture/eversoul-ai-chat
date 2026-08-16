use crate::infrastructure::i18n::pick;
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

#[derive(Debug, Clone, Serialize)]
pub struct SyncError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for SyncError {}

impl SyncError {
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
}
