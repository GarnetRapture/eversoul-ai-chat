use crate::infrastructure::i18n::pick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: String,
    pub title: String,
    pub persona_id: Option<String>,
    pub session_started_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub room_id: String,
    pub persona_id: Option<String>,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub room_id: String,
    pub content: String,
    pub persona_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub title: String,
    pub persona_id: Option<String>,
}

/// `code`는 프론트엔드 프로그래밍적 분기용, `message`는 SettingsManager의
/// 현재 언어(ko/en/zh_cn/zh_tw)로 이미 렌더링된 텍스트다. Tauri IPC로 그대로
/// 직렬화되어 프론트엔드 catch(err).message로 표시되므로 한국어 하드코딩 금지.
#[derive(Debug, Clone, Serialize)]
pub struct ChatError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for ChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ChatError {}

impl ChatError {
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

    pub fn llm_engine_not_loaded(language: &str) -> Self {
        Self {
            code: "llm_engine_not_loaded",
            message: pick(
                language,
                "로컬 LLM 엔진이 실행되지 않았습니다.".to_string(),
                "The local LLM engine is not running.".to_string(),
                "本地 LLM 引擎未运行。".to_string(),
            ),
        }
    }

    pub fn llm_inference_failed(language: &str, detail: &str) -> Self {
        Self {
            code: "llm_inference_failed",
            message: pick(
                language,
                format!("로컬 LLM 추론에 실패했습니다: {detail}"),
                format!("Local LLM inference failed: {detail}"),
                format!("本地 LLM 推理失败：{detail}"),
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
