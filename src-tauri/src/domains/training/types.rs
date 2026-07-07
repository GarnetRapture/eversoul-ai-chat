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
