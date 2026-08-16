use crate::infrastructure::i18n::pick;
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
    pub external_api: ExternalApiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiSettings {
    pub enabled: bool,
    pub base_url: String,
    pub api_key_configured: bool,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiConfigRequest {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiTestResult {
    pub ok: bool,
    pub message: String,
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

/// `code`는 프론트엔드 프로그래밍적 분기용, `message`는 SettingsManager의
/// 현재 언어(ko/en/zh_cn/zh_tw)로 이미 렌더링된 텍스트다. Tauri IPC로 그대로
/// 직렬화되어 프론트엔드 catch(err).message로 표시되므로 한국어 하드코딩 금지.
#[derive(Debug, Clone, Serialize)]
pub struct SettingsError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for SettingsError {}

impl SettingsError {
    pub fn io(language: &str, detail: &str) -> Self {
        Self {
            code: "io",
            message: pick(
                language,
                format!("설정 파일 접근 실패: {detail}"),
                format!("Failed to access the settings file: {detail}"),
                format!("设置文件访问失败：{detail}"),
            ),
        }
    }

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

    pub fn validation(language: &str, detail: &str) -> Self {
        Self {
            code: "validation",
            message: pick(
                language,
                format!("지원하지 않는 언어 설정: {detail}"),
                format!("Unsupported setting value: {detail}"),
                format!("不支持的设置值：{detail}"),
            ),
        }
    }
}
