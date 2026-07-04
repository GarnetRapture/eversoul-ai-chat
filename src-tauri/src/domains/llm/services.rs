use std::path::Path;
use crate::infrastructure::llm::LlmEngine;
use super::types::{LlmStatus, LlmInferResponse};

pub struct LlmService;

impl LlmService {
    /// 로컬 경로로부터 LLM Qwen 모델을 인스턴스화하고 가동한다.
    pub fn load_engine(app_root: &Path) -> Result<LlmEngine, String> {
        LlmEngine::load(app_root).map_err(|e| e.to_string())
    }

    /// 가동 중인 로컬 엔진 인스턴스를 사용해 실시간 추론을 수행한다.
    pub fn run_inference(
        engine: &LlmEngine,
        prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmInferResponse, String> {
        let start = std::time::Instant::now();
        let text = engine.infer(prompt, max_tokens).map_err(|e| e.to_string())?;
        let time_taken_ms = start.elapsed().as_millis() as u64;

        Ok(LlmInferResponse { text, time_taken_ms })
    }
}
