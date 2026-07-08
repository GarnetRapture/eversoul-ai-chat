use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_persona_id: Option<String>,
    pub active_style_id: Option<String>,
    pub language: String,
    pub language_configured: bool,
    pub inference_mode: String,
    pub api_provider: Option<String>,
    pub api_key: Option<String>,
    pub performance_tier: String,
    pub performance_configured: bool,
    pub setup_stage: String,
    pub show_reasoning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub physical_core_count: usize,
    pub logical_core_count: usize,
    pub total_memory_mb: u64,
    pub recommended_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
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
    #[error("지원하지 않는 언어 설정: {0}")]
    Validation(String),
}
