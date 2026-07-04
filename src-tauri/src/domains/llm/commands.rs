use super::services::{LlmLoadError, LlmService};
use super::types::{LlmError, LlmInferResponse, LlmStatus};
use crate::infrastructure::llm::LlmEngine;
use crate::infrastructure::llm::LlmError as InfraLlmError;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub struct LlmState(pub Mutex<Option<LlmEngine>>);

fn map_engine_error(err: InfraLlmError) -> LlmError {
    match err {
        InfraLlmError::ModelFileNotFound { path } => LlmError::ModelFileNotFound {
            path: path.to_string_lossy().to_string(),
        },
        InfraLlmError::BackendInit(msg) => LlmError::BackendInit(msg),
        InfraLlmError::ModelLoad(msg) => LlmError::ModelLoad(msg),
        InfraLlmError::ContextCreate(msg) => LlmError::ContextCreate(msg),
        InfraLlmError::Tokenize(msg) => LlmError::Tokenize(msg),
        InfraLlmError::Infer(msg) => LlmError::Infer(msg),
    }
}

fn map_load_error(err: LlmLoadError) -> LlmError {
    match err {
        LlmLoadError::ModelFileNotFound(paths) => LlmError::ModelFileNotFound {
            path: paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join(", "),
        },
        LlmLoadError::EngineError(e) => map_engine_error(e),
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

    if let Some(ref engine) = *engine_lock {
        return Ok(LlmStatus {
            is_loaded: true,
            model_path: Some(engine.model_path().to_string_lossy().to_string()),
            error_message: None,
        });
    }

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
            Err(map_load_error(e))
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
        LlmService::run_inference(engine, &prompt, max_tokens).map_err(map_engine_error)
    } else {
        Err(LlmError::EngineNotLoaded)
    }
}
