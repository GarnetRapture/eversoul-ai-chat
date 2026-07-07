use super::types::LlmInferResponse;
use crate::infrastructure::hardware::InferenceProfile;
use crate::infrastructure::llm::validation::{validate_model_file, ModelFileValidation};
use crate::infrastructure::llm::worker::LlmWorkerHandle;
use crate::infrastructure::llm::LlmError as InfraLlmError;
use crate::infrastructure::llm::get_model_relative_path;
use crate::startup_debug_log;
use std::path::{Path, PathBuf};

pub struct LlmService;

pub enum LlmLoadError {
    ModelFileNotFound(Vec<PathBuf>),

    EngineError(InfraLlmError),
}

impl LlmService {
    pub fn model_roots(app_root: &Path) -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                candidates.push(exe_dir.to_path_buf());
            }
        }

        candidates.push(app_root.to_path_buf());

        if let Ok(current_dir) = std::env::current_dir() {
            candidates.push(current_dir.clone());
            if let Some(parent) = current_dir.parent() {
                candidates.push(parent.to_path_buf());
            }
        }

        candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));

        let mut unique = Vec::new();
        for candidate in candidates {
            let normalized = candidate
                .canonicalize()
                .unwrap_or_else(|_| candidate.clone());
            if !unique.contains(&normalized) {
                unique.push(normalized);
            }
        }
        unique
    }

    pub fn model_destination_path(app_root: &Path, active_model: &str) -> PathBuf {
        let relative_path = get_model_relative_path(active_model);
        for root in Self::model_roots(app_root) {
            let model_path = root.join(relative_path);
            if model_path.exists() {
                return model_path;
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join(relative_path);
            }
        }

        app_root.join(relative_path)
    }

    pub fn load_engine(
        app_root: &Path,
        adapters_dir: PathBuf,
        profile: InferenceProfile,
        active_model: &str,
    ) -> Result<LlmWorkerHandle, LlmLoadError> {
        startup_debug_log("llm_service:load_engine:start");
        let mut attempted_paths = Vec::new();
        let relative_path = get_model_relative_path(active_model);
        for root in Self::model_roots(app_root) {
            let model_path = root.join(relative_path);
            if attempted_paths.contains(&model_path) {
                continue;
            }

            attempted_paths.push(model_path.clone());
            startup_debug_log(&format!(
                "llm_service:load_engine:check_path:{}",
                model_path.display()
            ));
            if model_path.exists() {
                startup_debug_log("llm_service:load_engine:model_found");
                startup_debug_log("llm_service:load_engine:validate:start");
                validate_model_file(&model_path).map_err(LlmLoadError::EngineError)?;
                startup_debug_log("llm_service:load_engine:validate:done");
                startup_debug_log("llm_service:load_engine:worker_load:start");
                return LlmWorkerHandle::load_and_spawn(root, adapters_dir, profile, relative_path)
                    .map_err(LlmLoadError::EngineError);
            }
        }

        startup_debug_log("llm_service:load_engine:model_not_found");
        Err(LlmLoadError::ModelFileNotFound(attempted_paths))
    }

    pub fn validate_model(app_root: &Path, active_model: &str) -> Result<ModelFileValidation, LlmLoadError> {
        startup_debug_log("llm_service:validate_model:start");
        let mut attempted_paths = Vec::new();
        let relative_path = get_model_relative_path(active_model);
        for root in Self::model_roots(app_root) {
            let model_path = root.join(relative_path);
            if attempted_paths.contains(&model_path) {
                continue;
            }
            attempted_paths.push(model_path.clone());
            if model_path.exists() {
                startup_debug_log("llm_service:validate_model:model_found");
                let result = validate_model_file(&model_path).map_err(LlmLoadError::EngineError);
                startup_debug_log("llm_service:validate_model:done");
                return result;
            }
        }
        startup_debug_log("llm_service:validate_model:model_not_found");
        Err(LlmLoadError::ModelFileNotFound(attempted_paths))
    }

    pub fn run_inference(
        handle: &LlmWorkerHandle,
        prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmInferResponse, InfraLlmError> {
        let start = std::time::Instant::now();
        let text = handle.infer(prompt, max_tokens, None)?;
        let time_taken_ms = start.elapsed().as_millis() as u64;

        Ok(LlmInferResponse {
            text,
            time_taken_ms,
        })
    }

    pub fn run_inference_with_request(
        handle: &LlmWorkerHandle,
        request_id: &str,
        prompt: &str,
        max_tokens: Option<u32>,
        persona_id: Option<&str>,
        stream: Option<(tauri::AppHandle, String, String)>,
    ) -> Result<LlmInferResponse, InfraLlmError> {
        let start = std::time::Instant::now();
        let text = handle.infer_with_request(request_id, prompt, max_tokens, persona_id, stream)?;
        let time_taken_ms = start.elapsed().as_millis() as u64;

        Ok(LlmInferResponse {
            text,
            time_taken_ms,
        })
    }
}
