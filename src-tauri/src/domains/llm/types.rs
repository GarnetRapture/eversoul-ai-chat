use crate::infrastructure::i18n::pick;
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

/// `code`는 프론트엔드 프로그래밍적 분기용, `message`는 SettingsManager의
/// 현재 언어(ko/en/zh_cn/zh_tw)로 이미 렌더링된 텍스트다. Tauri IPC로 그대로
/// 직렬화되어 프론트엔드 catch(err).message로 표시되므로 한국어 하드코딩 금지.
#[derive(Debug, Clone, Serialize)]
pub struct LlmError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for LlmError {}

impl LlmError {
    pub fn model_file_not_found(language: &str, path: &str) -> Self {
        Self {
            code: "model_file_not_found",
            message: pick(
                language,
                format!("모델 파일을 찾을 수 없습니다: {path}"),
                format!("Model file not found: {path}"),
                format!("找不到模型文件：{path}"),
            ),
        }
    }

    pub fn backend_init(language: &str, detail: &str) -> Self {
        Self {
            code: "backend_init",
            message: pick(
                language,
                format!("LLM 엔진 백엔드 초기화 실패: {detail}"),
                format!("Failed to initialize the LLM engine backend: {detail}"),
                format!("LLM 引擎后端初始化失败：{detail}"),
            ),
        }
    }

    pub fn model_load(language: &str, detail: &str) -> Self {
        Self {
            code: "model_load",
            message: pick(
                language,
                format!("모델 로딩 실패: {detail}"),
                format!("Failed to load the model: {detail}"),
                format!("模型加载失败：{detail}"),
            ),
        }
    }

    pub fn model_download(language: &str, detail: &str) -> Self {
        Self {
            code: "model_download",
            message: pick(
                language,
                format!("모델 다운로드 실패: {detail}"),
                format!("Model download failed: {detail}"),
                format!("模型下载失败：{detail}"),
            ),
        }
    }

    pub fn context_create(language: &str, detail: &str) -> Self {
        Self {
            code: "context_create",
            message: pick(
                language,
                format!("컨텍스트 생성 실패: {detail}"),
                format!("Failed to create the context: {detail}"),
                format!("上下文创建失败：{detail}"),
            ),
        }
    }

    pub fn tokenize(language: &str, detail: &str) -> Self {
        Self {
            code: "tokenize",
            message: pick(
                language,
                format!("토큰화 실패: {detail}"),
                format!("Tokenization failed: {detail}"),
                format!("分词失败：{detail}"),
            ),
        }
    }

    pub fn infer(language: &str, detail: &str) -> Self {
        Self {
            code: "infer",
            message: pick(
                language,
                format!("추론 실패: {detail}"),
                format!("Inference failed: {detail}"),
                format!("推理失败：{detail}"),
            ),
        }
    }

    pub fn engine_not_loaded(language: &str) -> Self {
        Self {
            code: "engine_not_loaded",
            message: pick(
                language,
                "엔진이 로드되지 않았습니다.".to_string(),
                "The engine is not loaded.".to_string(),
                "引擎尚未加载。".to_string(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableLocalModel {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub size_bytes: u64,
    pub is_downloaded: bool,
}
