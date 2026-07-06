use super::services::{LlmLoadError, LlmService};
use super::types::{
    LlmError, LlmInferResponse, LlmModelValidation, LlmRequestStatus, LlmSessionGenerationStats,
    LlmSessionStatus, LlmStatus, LlmStreamInferRequest,
};
use crate::domains::settings::commands::SettingsState;
use crate::domains::training::commands::TrainingState;
use crate::infrastructure::hardware::{HardwareDetector, PerformanceTier};
use crate::infrastructure::llm::scheduler::LlmRequestStatus as InfraRequestStatus;
use crate::infrastructure::llm::validation::ModelFileValidation;
use crate::infrastructure::llm::worker::LlmWorkerHandle;
use crate::infrastructure::llm::worker::WorkerSessionStatus;
use crate::infrastructure::llm::LlmError as InfraLlmError;
use crate::startup_debug_log;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub struct LlmState(pub Mutex<Option<LlmWorkerHandle>>);

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

fn map_session_status(status: WorkerSessionStatus) -> LlmSessionStatus {
    LlmSessionStatus {
        persona_id: status.persona_id,
        cached_tokens: status.cached_tokens,
        lora_adapter_mounted: status.lora_adapter_mounted,
        last_access: status.last_access,
        last_generation: status
            .last_generation
            .map(|stats| LlmSessionGenerationStats {
                prompt_tokens: stats.prompt_tokens,
                cached_tokens: stats.cached_tokens,
                generated_tokens: stats.generated_tokens,
                reused_prefix_tokens: stats.reused_prefix_tokens,
                truncated_prompt_tokens: stats.truncated_prompt_tokens,
                cache_reset: stats.cache_reset,
            }),
    }
}

fn map_request_status(status: InfraRequestStatus) -> LlmRequestStatus {
    LlmRequestStatus {
        request_id: status.request_id,
        persona_id: status.persona_id,
        state: status.state,
        prompt_tokens: status.prompt_tokens,
        generated_tokens: status.generated_tokens,
        reused_prefix_tokens: status.reused_prefix_tokens,
        truncated_prompt_tokens: status.truncated_prompt_tokens,
        cache_reset: status.cache_reset,
        error_message: status.error_message,
    }
}

fn map_model_validation(validation: ModelFileValidation) -> LlmModelValidation {
    LlmModelValidation {
        path: validation.path,
        size_bytes: validation.size_bytes,
        sha256: validation.sha256,
        sidecar_sha256: validation.sidecar_sha256,
        hash_matches_sidecar: validation.hash_matches_sidecar,
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_load(
    app_handle: AppHandle,
    llm_state: State<'_, LlmState>,
    training_state: State<'_, TrainingState>,
    settings_state: State<'_, SettingsState>,
) -> Result<LlmStatus, LlmError> {
    startup_debug_log("command:llm_load:start");
    let mut engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;
    startup_debug_log("command:llm_load:state_locked");

    if let Some(ref handle) = *engine_lock {
        startup_debug_log("command:llm_load:already_loaded");
        return Ok(LlmStatus {
            is_loaded: true,
            model_path: Some(handle.model_path().to_string_lossy().to_string()),
            error_message: None,
        });
    }

    let app_root = app_handle
        .path()
        .resource_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    startup_debug_log("command:llm_load:app_root_ready");

    let adapters_dir = training_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?
        .clone();
    startup_debug_log("command:llm_load:adapters_ready");

    let tier_str = settings_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?
        .get_performance_tier();
    startup_debug_log("command:llm_load:tier_ready");
    let hardware = HardwareDetector::detect();
    let profile = HardwareDetector::inference_profile_for(
        PerformanceTier::from_str(&tier_str),
        hardware.physical_core_count,
    );
    startup_debug_log("command:llm_load:profile_ready");

    match LlmService::load_engine(&app_root, adapters_dir, profile) {
        Ok(handle) => {
            startup_debug_log("command:llm_load:engine_loaded");
            let model_path_str = handle.model_path().to_string_lossy().to_string();
            *engine_lock = Some(handle);
            startup_debug_log("command:llm_load:done");

            Ok(LlmStatus {
                is_loaded: true,
                model_path: Some(model_path_str),
                error_message: None,
            })
        }
        Err(e) => {
            *engine_lock = None;
            startup_debug_log("command:llm_load:error");
            Err(map_load_error(e))
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_unload(llm_state: State<'_, LlmState>) -> Result<(), LlmError> {
    startup_debug_log("command:llm_unload:start");
    let mut engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    *engine_lock = None;
    startup_debug_log("command:llm_unload:done");
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_status(llm_state: State<'_, LlmState>) -> Result<LlmStatus, LlmError> {
    startup_debug_log("command:llm_status:start");
    let engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    if let Some(ref handle) = *engine_lock {
        startup_debug_log("command:llm_status:loaded");
        Ok(LlmStatus {
            is_loaded: true,
            model_path: Some(handle.model_path().to_string_lossy().to_string()),
            error_message: None,
        })
    } else {
        startup_debug_log("command:llm_status:not_loaded");
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
    startup_debug_log("command:llm_infer:start");
    let engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    if let Some(ref handle) = *engine_lock {
        let result = LlmService::run_inference(handle, &prompt, max_tokens).map_err(map_engine_error);
        startup_debug_log("command:llm_infer:done");
        result
    } else {
        startup_debug_log("command:llm_infer:not_loaded");
        Err(LlmError::EngineNotLoaded)
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_infer_stream(
    app_handle: AppHandle,
    llm_state: State<'_, LlmState>,
    request: LlmStreamInferRequest,
) -> Result<LlmInferResponse, LlmError> {
    startup_debug_log("command:llm_infer_stream:start");
    let engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    if let Some(ref handle) = *engine_lock {
        let result = LlmService::run_inference_with_request(
            handle,
            &request.request_id,
            &request.prompt,
            request.max_tokens,
            request.persona_id.as_deref(),
            Some((app_handle, request.token_event, request.done_event)),
        )
        .map_err(map_engine_error);
        startup_debug_log("command:llm_infer_stream:done");
        result
    } else {
        startup_debug_log("command:llm_infer_stream:not_loaded");
        Err(LlmError::EngineNotLoaded)
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_cancel_request(
    llm_state: State<'_, LlmState>,
    request_id: String,
) -> Result<bool, LlmError> {
    startup_debug_log("command:llm_cancel_request:start");
    let engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    let result = engine_lock
        .as_ref()
        .is_some_and(|handle| handle.cancel_request(&request_id));
    startup_debug_log("command:llm_cancel_request:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_active_sessions(llm_state: State<'_, LlmState>) -> Result<Vec<String>, LlmError> {
    startup_debug_log("command:llm_active_sessions:start");
    let engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    let result = engine_lock
        .as_ref()
        .map(|handle| handle.active_sessions())
        .unwrap_or_default();
    startup_debug_log("command:llm_active_sessions:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_session_statuses(
    llm_state: State<'_, LlmState>,
) -> Result<Vec<LlmSessionStatus>, LlmError> {
    startup_debug_log("command:llm_session_statuses:start");
    let engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    let result = engine_lock
        .as_ref()
        .map(|handle| {
            handle
                .session_statuses()
                .into_iter()
                .map(map_session_status)
                .collect()
        })
        .unwrap_or_default();
    startup_debug_log("command:llm_session_statuses:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_request_statuses(
    llm_state: State<'_, LlmState>,
) -> Result<Vec<LlmRequestStatus>, LlmError> {
    startup_debug_log("command:llm_request_statuses:start");
    let engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;

    let result = engine_lock
        .as_ref()
        .map(|handle| {
            handle
                .request_statuses()
                .into_iter()
                .map(map_request_status)
                .collect()
        })
        .unwrap_or_default();
    startup_debug_log("command:llm_request_statuses:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_verify_model(app_handle: AppHandle) -> Result<LlmModelValidation, LlmError> {
    startup_debug_log("command:llm_verify_model:start");
    let app_root = app_handle
        .path()
        .resource_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    let result = LlmService::validate_model(&app_root)
        .map(map_model_validation)
        .map_err(map_load_error);
    startup_debug_log("command:llm_verify_model:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn llm_self_test(
    app_handle: AppHandle,
    llm_state: State<'_, LlmState>,
    training_state: State<'_, TrainingState>,
    settings_state: State<'_, SettingsState>,
) -> Result<LlmInferResponse, LlmError> {
    startup_debug_log("command:llm_self_test:start");
    let status = llm_load(
        app_handle,
        llm_state.clone(),
        training_state,
        settings_state,
    )?;
    if !status.is_loaded {
        startup_debug_log("command:llm_self_test:not_loaded");
        return Err(LlmError::EngineNotLoaded);
    }
    let engine_lock = llm_state
        .inner()
        .0
        .lock()
        .map_err(|e| LlmError::Unknown(e.to_string()))?;
    if let Some(ref handle) = *engine_lock {
        let result = LlmService::run_inference(
            handle,
            "<|im_start|>system\n한국어로 짧게 답하십시오.<|im_end|>\n<|im_start|>user\n테스트<|im_end|>\n<|im_start|>assistant\n",
            Some(8),
        )
        .map_err(map_engine_error);
        startup_debug_log("command:llm_self_test:done");
        result
    } else {
        startup_debug_log("command:llm_self_test:missing_handle");
        Err(LlmError::EngineNotLoaded)
    }
}
