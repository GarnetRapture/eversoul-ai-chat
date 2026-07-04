use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSummary {
    pub persona_id: String,
    pub examples_used: usize,
    pub steps: usize,
    pub final_loss: f32,
    pub adapter_path: String,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum TrainingError {
    #[error("데이터베이스 오류: {0}")]
    Database(String),
    #[error("학습할 대화 데이터가 부족합니다: {0}")]
    InsufficientData(String),
    #[error("LoRA 학습 실패: {0}")]
    TrainingFailed(String),
}
