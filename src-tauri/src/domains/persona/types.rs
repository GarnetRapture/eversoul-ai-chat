use crate::infrastructure::i18n::pick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaConfig {
    pub id: String,
    pub name: String,
    pub name_en: String,
    pub grade: String,
    pub race: String,
    pub class: String,
    pub sub_class: String,
    pub system_prompt: String,
    pub greeting: String,
    pub raw_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaLocalizedPrompt {
    pub persona_id: String,
    pub language: String,
    pub localized_name: String,
    pub assembled_prompt: String,
    pub source_updated_at: String,
    pub cached_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondRankingEntry {
    pub persona_id: String,
    pub name: String,
    pub name_en: String,
    pub message_count: usize,
    pub memory_count: usize,
    pub bond_score: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamiliarityEntry {
    pub persona_id: String,
    pub name: String,
    pub name_en: String,
    pub message_count: usize,
    pub memory_count: usize,
    pub familiarity_score: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePersonaRequest {
    pub id: String,
    pub system_prompt: String,
    pub greeting: String,
}

/// `code`는 프론트엔드 프로그래밍적 분기용, `message`는 SettingsManager의
/// 현재 언어(ko/en/zh_cn/zh_tw)로 이미 렌더링된 텍스트다. Tauri IPC로 그대로
/// 직렬화되어 프론트엔드 catch(err).message로 표시되므로 한국어 하드코딩 금지.
#[derive(Debug, Clone, Serialize)]
pub struct PersonaError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for PersonaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for PersonaError {}

impl PersonaError {
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

    pub fn archive(language: &str, detail: &str) -> Self {
        Self {
            code: "archive",
            message: pick(
                language,
                format!("압축 아카이브 오류: {detail}"),
                format!("Archive error: {detail}"),
                format!("压缩存档错误：{detail}"),
            ),
        }
    }

    pub fn not_found(language: &str, id: &str) -> Self {
        Self {
            code: "not_found",
            message: pick(
                language,
                format!("페르소나 프로필을 찾을 수 없습니다: {id}"),
                format!("Persona profile not found: {id}"),
                format!("找不到精灵资料：{id}"),
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
