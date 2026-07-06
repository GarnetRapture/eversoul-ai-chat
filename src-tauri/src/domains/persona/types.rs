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

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum PersonaError {
    #[error("데이터베이스 오류: {0}")]
    Database(String),
    #[error("압축 아카이브 오류: {0}")]
    Archive(String),
    #[error("페르소나 프로필을 찾을 수 없습니다: {0}")]
    NotFound(String),
    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}
