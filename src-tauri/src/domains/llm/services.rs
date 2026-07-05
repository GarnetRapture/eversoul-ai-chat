use super::types::LlmInferResponse;
use crate::infrastructure::llm::LlmEngine;
use crate::infrastructure::llm::LlmError as InfraLlmError;
use crate::infrastructure::llm::MODEL_RELATIVE_PATH;
use std::path::{Path, PathBuf};

pub struct LlmService;

pub enum LlmLoadError {
    ModelFileNotFound(Vec<PathBuf>),

    EngineError(InfraLlmError),
}

impl LlmService {
    pub fn load_engine(app_root: &Path, adapters_dir: PathBuf) -> Result<LlmEngine, LlmLoadError> {
        let mut candidates = vec![app_root.to_path_buf()];

        if let Ok(current_dir) = std::env::current_dir() {
            candidates.push(current_dir.clone());
            if let Some(parent) = current_dir.parent() {
                candidates.push(parent.to_path_buf());
            }
        }

        let mut attempted_paths = Vec::new();
        for root in candidates {
            let model_path = root.join(MODEL_RELATIVE_PATH);
            if attempted_paths.contains(&model_path) {
                continue;
            }

            attempted_paths.push(model_path.clone());
            if model_path.exists() {
                return LlmEngine::load(&root, adapters_dir).map_err(LlmLoadError::EngineError);
            }
        }

        Err(LlmLoadError::ModelFileNotFound(attempted_paths))
    }

    pub fn run_inference(
        engine: &LlmEngine,
        prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmInferResponse, InfraLlmError> {
        let start = std::time::Instant::now();
        let text = engine.infer(prompt, max_tokens, None)?;
        let time_taken_ms = start.elapsed().as_millis() as u64;

        Ok(LlmInferResponse {
            text,
            time_taken_ms,
        })
    }
}
