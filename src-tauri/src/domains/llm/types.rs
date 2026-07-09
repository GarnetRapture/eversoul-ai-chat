use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStatus {
    pub is_loaded: bool,
    pub model_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmLoadRequest {
    pub model_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInferRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInferResponse {
    pub text: String,
    pub time_taken_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStreamInferRequest {
    pub request_id: String,
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub persona_id: Option<String>,
    pub token_event: String,
    pub done_event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModelValidation {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub sidecar_sha256: Option<String>,
    pub hash_matches_sidecar: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequestStatus {
    pub request_id: String,
    pub persona_id: Option<String>,
    pub state: String,
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub reused_prefix_tokens: usize,
    pub truncated_prompt_tokens: usize,
    pub cache_reset: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSessionGenerationStats {
    pub prompt_tokens: usize,
    pub cached_tokens: usize,
    pub generated_tokens: usize,
    pub reused_prefix_tokens: usize,
    pub truncated_prompt_tokens: usize,
    pub cache_reset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSessionStatus {
    pub persona_id: String,
    pub cached_tokens: usize,
    pub lora_adapter_mounted: bool,
    pub last_access: u64,
    pub last_generation: Option<LlmSessionGenerationStats>,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum LlmError {
    #[error("모델 파일을 찾을 수 없습니다: {path}")]
    ModelFileNotFound { path: String },
    #[error("LLM 엔진 백엔드 초기화 실패: {0}")]
    BackendInit(String),
    #[error("모델 로딩 실패: {0}")]
    ModelLoad(String),
    #[error("모델 다운로드 실패: {0}")]
    ModelDownload(String),
    #[error("컨텍스트 생성 실패: {0}")]
    ContextCreate(String),
    #[error("토큰 파싱 실패: {0}")]
    Tokenize(String),
    #[error("추론 실패: {0}")]
    Infer(String),
    #[error("엔진이 로드되지 않았습니다.")]
    EngineNotLoaded,
    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableLocalModel {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub size_bytes: u64,
    pub is_downloaded: bool,
}
