use super::services::LlmService;
use super::types::{LlmError, LlmInferResponse, LlmStatus};
use crate::infrastructure::llm::LlmEngine;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub struct LlmState(pub Mutex<Option<LlmEngine>>);

fn map_infra_error(err: String) -> LlmError {
    if err.contains("ModelFileNotFound") {
        LlmError::ModelFileNotFound {
            path: "ai/model/qwen25-3b-korean-Q4_K_M.gguf".to_string(),
        }
    } else if err.contains("BackendInit") {
        LlmError::BackendInit(err)
    } else if err.contains("ModelLoad") {
        LlmError::ModelLoad(err)
    } else if err.contains("ContextCreate") {
        LlmError::ContextCreate(err)
    } else if err.contains("Tokenize") {
        LlmError::Tokenize(err)
    } else {
        LlmError::Infer(err)
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_load(
    app_handle: AppHandle,
    llm_state: State<'_, LlmState>,
) -> Result<LlmStatus, LlmError> {
    let mut engine_lock = llm_state
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    // 개발 모드와 빌드 배포 모드 양측 경로에 모두 호환되도록 기준 경로 수립
    let app_root = app_handle
        .path()
        .resource_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    match LlmService::load_engine(&app_root) {
        Ok(engine) => {
            let model_path_str = engine.model_path().to_string_lossy().to_string();
            *engine_lock = Some(engine);

            Ok(LlmStatus {
                is_loaded: true,
                model_path: Some(model_path_str),
                error_message: None,
            })
        }
        Err(e) => {
            *engine_lock = None;
            Err(map_infra_error(e))
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_status(llm_state: State<'_, LlmState>) -> Result<LlmStatus, LlmError> {
    let engine_lock = llm_state
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    if let Some(ref engine) = *engine_lock {
        Ok(LlmStatus {
            is_loaded: true,
            model_path: Some(engine.model_path().to_string_lossy().to_string()),
            error_message: None,
        })
    } else {
        Ok(LlmStatus {
            is_loaded: false,
            model_path: None,
            error_message: None,
        })
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_infer(
    llm_state: State<'_, LlmState>,
    prompt: String,
    max_tokens: Option<u32>,
) -> Result<LlmInferResponse, LlmError> {
    let engine_lock = llm_state
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    if let Some(ref engine) = *engine_lock {
        LlmService::run_inference(engine, &prompt, max_tokens).map_err(map_infra_error)
    } else {
        Err(LlmError::EngineNotLoaded)
    }
}
