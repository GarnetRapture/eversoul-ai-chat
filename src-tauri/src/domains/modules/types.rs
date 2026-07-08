use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source_path: Option<String>,
    pub enabled: bool,
    pub prompt: String,
    #[serde(default)]
    pub controls: Vec<ModuleControl>,
    pub lorebook_count: usize,
    pub regex_count: usize,
    pub trigger_count: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleControl {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub value: String,
    #[serde(default)]
    pub options: Vec<ModuleControlOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleControlOption {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleStore {
    pub modules: Vec<ImportedModule>,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum ModuleError {
    #[error("모듈 파일 접근 실패: {0}")]
    Io(String),
    #[error("지원하지 않는 모듈 파일입니다: {0}")]
    InvalidFormat(String),
    #[error("모듈 저장 실패: {0}")]
    Storage(String),
}
