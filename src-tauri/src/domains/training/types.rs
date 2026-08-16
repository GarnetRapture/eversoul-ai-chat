use crate::infrastructure::i18n::pick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSummary {
    pub persona_id: String,
    pub examples_used: usize,
    pub steps: usize,
    pub final_loss: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingProgress {
    pub persona_id: String,
    pub step: usize,
    pub total_steps: usize,
    pub loss: f32,
}

/// `code`는 프론트엔드 프로그래밍적 분기용, `message`는 SettingsManager의
/// 현재 언어(ko/en/zh_tw/zh_cn)로 이미 렌더링된 텍스트다. 백엔드 로그와
/// 프론트엔드 표시 모두 이 message를 그대로 쓴다 — 한국어 하드코딩 금지.
#[derive(Debug, Clone, Serialize)]
pub struct TrainingError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for TrainingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for TrainingError {}

impl TrainingError {
    pub fn db_lock(language: &str) -> Self {
        Self {
            code: "db_lock",
            message: pick(
                language,
                "데이터베이스 락 획득에 실패했습니다.".to_string(),
                "Failed to acquire the database lock.".to_string(),
                "获取数据库锁失败。".to_string(),
            ),
        }
    }

    pub fn persona_not_found(language: &str, persona_id: &str) -> Self {
        Self {
            code: "persona_not_found",
            message: pick(
                language,
                format!("정령을 찾을 수 없습니다: {persona_id}"),
                format!("Soul not found: {persona_id}"),
                format!("找不到精灵：{persona_id}"),
            ),
        }
    }

    pub fn query_failed(language: &str) -> Self {
        Self {
            code: "query_failed",
            message: pick(
                language,
                "대화 기록 조회에 실패했습니다.".to_string(),
                "Failed to load conversation history.".to_string(),
                "加载对话记录失败。".to_string(),
            ),
        }
    }

    pub fn insufficient_data(language: &str, required: usize, current: usize) -> Self {
        Self {
            code: "insufficient_data",
            message: pick(
                language,
                format!("학습을 위한 대화가 부족합니다 (최소 {required}개 필요, 현재 {current}개)."),
                format!("Not enough conversation data to train (needs at least {required}, has {current})."),
                format!("训练所需对话数据不足（至少需要 {required} 条，当前 {current} 条）。"),
            ),
        }
    }

    pub fn architecture_mismatch(language: &str, expected: &str, found: &str) -> Self {
        Self {
            code: "architecture_mismatch",
            message: pick(
                language,
                format!("베이스 모델 아키텍처가 일치하지 않습니다 (필요: {expected}, GGUF: {found})."),
                format!("Base model architecture mismatch (expected {expected}, GGUF has {found})."),
                format!("基础模型架构不匹配（需要 {expected}，GGUF 为 {found}）。"),
            ),
        }
    }

    pub fn base_model_load_failed(language: &str, detail: &str) -> Self {
        Self {
            code: "base_model_load_failed",
            message: pick(
                language,
                format!("베이스 GGUF 모델 로드에 실패했습니다: {detail}"),
                format!("Failed to load the base GGUF model: {detail}"),
                format!("加载基础 GGUF 模型失败：{detail}"),
            ),
        }
    }

    pub fn tokenizer_load_failed(language: &str, detail: &str) -> Self {
        Self {
            code: "tokenizer_load_failed",
            message: pick(
                language,
                format!("토크나이저 로드에 실패했습니다: {detail}"),
                format!("Failed to load the tokenizer: {detail}"),
                format!("加载分词器失败：{detail}"),
            ),
        }
    }

    pub fn thread_panic(language: &str, detail: &str) -> Self {
        Self {
            code: "thread_panic",
            message: pick(
                language,
                format!("학습 스레드가 예기치 않게 종료되었습니다: {detail}"),
                format!("The training thread panicked unexpectedly: {detail}"),
                format!("训练线程意外终止：{detail}"),
            ),
        }
    }

    pub fn training_failed(language: &str, detail: &str) -> Self {
        Self {
            code: "training_failed",
            message: pick(
                language,
                format!("LoRA 학습에 실패했습니다: {detail}"),
                format!("LoRA training failed: {detail}"),
                format!("LoRA 训练失败：{detail}"),
            ),
        }
    }

    pub fn state_lock(language: &str) -> Self {
        Self {
            code: "state_lock",
            message: pick(
                language,
                "학습 상태 락 획득에 실패했습니다.".to_string(),
                "Failed to acquire the training state lock.".to_string(),
                "获取训练状态锁失败。".to_string(),
            ),
        }
    }
}
