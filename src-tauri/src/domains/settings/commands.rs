use super::services::SettingsService;
use super::types::{
    AppSettings, ExternalApiConfigRequest, ExternalApiTestResult, HardwareProfile, ResetSummary,
    SettingsError, SetupProgress,
};
use crate::domains::auth::commands::DbState;
use crate::domains::llm::commands::{CacheState, LlmState};
use crate::domains::llm::services::LlmService;
use crate::domains::persona::repositories::PersonaRepository;
use crate::domains::persona::services::PersonaService;
use crate::domains::training::commands::TrainingState;
use crate::infrastructure::api_key::{ApiKeyController, CONNECTION_TEST_MAX_TOKENS};
use crate::infrastructure::compress::PersonaLoader;
use crate::infrastructure::external_ai::infer_chat;
use crate::infrastructure::hardware::{HardwareDetector, PerformanceTier};
use crate::infrastructure::settings::SettingsManager;
use crate::startup_debug_log;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct SettingsState(pub Mutex<SettingsManager>);
pub struct ApiKeyState(pub Mutex<ApiKeyController>);

fn lock_settings_fallback<'a>(
    settings_state: &'a State<'a, SettingsState>,
) -> Result<std::sync::MutexGuard<'a, SettingsManager>, SettingsError> {
    settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::io("ko", &e.to_string()))
}

fn lock_api_keys_fallback<'a>(
    api_key_state: &'a State<'a, ApiKeyState>,
) -> Result<std::sync::MutexGuard<'a, ApiKeyController>, SettingsError> {
    api_key_state
        .0
        .lock()
        .map_err(|e| SettingsError::io("ko", &e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_get(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_get:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let result = SettingsService::get_settings(&settings, &api_keys);
    startup_debug_log("command:settings_get:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_reset(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
) -> Result<ResetSummary, SettingsError> {
    startup_debug_log("command:settings_reset:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let language = settings.get_language();
    let conn = db_state
        .0
        .get()
        .map_err(|e| SettingsError::database(&language, &e.to_string()))?;
    let result = SettingsService::reset_all(&conn, &settings, &api_keys);
    startup_debug_log("command:settings_reset:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_language(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    language: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_language:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let current_language = settings.get_language();
    let conn = db_state
        .0
        .get()
        .map_err(|e| SettingsError::database(&current_language, &e.to_string()))?;
    let result = SettingsService::set_language(&conn, &settings, &api_keys, &language);
    startup_debug_log("command:settings_set_language:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_performance_tier(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    tier: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_performance_tier:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let result = SettingsService::set_performance_tier(&settings, &api_keys, &tier);
    startup_debug_log("command:settings_set_performance_tier:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_inference_mode(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    mode: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_inference_mode:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let result = SettingsService::set_inference_mode(&settings, &api_keys, &mode);
    startup_debug_log("command:settings_set_inference_mode:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_api_provider(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    provider: Option<String>,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_api_provider:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let result = SettingsService::set_api_provider(&settings, &api_keys, provider.as_deref());
    startup_debug_log("command:settings_set_api_provider:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_api_key(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    key: Option<String>,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_api_key:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let result = SettingsService::set_api_key(&settings, &api_keys, key.as_deref());
    startup_debug_log("command:settings_set_api_key:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_setup_stage(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    stage: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_setup_stage:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let result = SettingsService::set_setup_stage(&settings, &api_keys, &stage);
    startup_debug_log("command:settings_set_setup_stage:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_detect_hardware() -> Result<HardwareProfile, SettingsError> {
    startup_debug_log("command:settings_detect_hardware:start");
    let result = SettingsService::detect_hardware();
    startup_debug_log("command:settings_detect_hardware:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_show_reasoning(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    show_reasoning: bool,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_show_reasoning:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let result = SettingsService::set_show_reasoning(&settings, &api_keys, show_reasoning);
    startup_debug_log("command:settings_set_show_reasoning:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_active_model(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    model: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_active_model:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let language = settings.get_language();
    settings
        .set_active_model(&model)
        .map_err(|e| SettingsError::io(&language, &e.to_string()))?;
    let result = SettingsService::get_settings(&settings, &api_keys);
    startup_debug_log("command:settings_set_active_model:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_external_api_config(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    request: ExternalApiConfigRequest,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_external_api_config:start");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    let result = SettingsService::set_external_api_config(&settings, &api_keys, &request);
    startup_debug_log("command:settings_set_external_api_config:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub async fn settings_test_external_api(
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
) -> Result<ExternalApiTestResult, SettingsError> {
    startup_debug_log("command:settings_test_external_api:start");
    let language = lock_settings_fallback(&settings_state)?.get_language();
    let config = {
        let api_keys = lock_api_keys_fallback(&api_key_state)?;
        api_keys.build_external_config()
    };

    let Some(config) = config else {
        startup_debug_log("command:settings_test_external_api:missing_key");
        return Ok(ExternalApiTestResult {
            ok: false,
            message: SettingsError::validation(&language, "external_api_key is not configured")
                .message,
        });
    };

    let result = match infer_chat(
        &config,
        "Reply with a short connection success message.",
        &[],
        CONNECTION_TEST_MAX_TOKENS,
    )
    .await
    {
        Ok(text) => ExternalApiTestResult {
            ok: true,
            message: text,
        },
        Err(err) => ExternalApiTestResult {
            ok: false,
            message: SettingsError::validation(&language, &err.to_string()).message,
        },
    };
    startup_debug_log("command:settings_test_external_api:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_complete_initial_setup(
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    api_key_state: State<'_, ApiKeyState>,
    llm_state: State<'_, LlmState>,
    cache_state: State<'_, CacheState>,
    training_state: State<'_, TrainingState>,
    language: String,
    inference_mode: String,
    api_provider: Option<String>,
    api_key: Option<String>,
    tier: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_complete_initial_setup:start");
    let conn = db_state
        .0
        .get()
        .map_err(|e| SettingsError::database(&language, &e.to_string()))?;
    startup_debug_log("command:settings_complete_initial_setup:db_locked");
    let settings = lock_settings_fallback(&settings_state)?;
    let api_keys = lock_api_keys_fallback(&api_key_state)?;
    startup_debug_log("command:settings_complete_initial_setup:settings_locked");

    SettingsService::set_language_without_warmup(&settings, &api_keys, &language)?;
    settings
        .set_inference_mode(&inference_mode)
        .map_err(|e| SettingsError::io(&language, &e.to_string()))?;
    if let Some(ref provider) = api_provider {
        api_keys
            .set_provider(Some(provider))
            .map_err(|e| SettingsError::io(&language, &e.to_string()))?;
    }
    if let Some(ref key) = api_key {
        api_keys
            .set_local_api_key(Some(key))
            .map_err(|e| SettingsError::io(&language, &e.to_string()))?;
    }
    SettingsService::set_performance_tier(&settings, &api_keys, &tier)?;
    SettingsService::set_setup_stage(&settings, &api_keys, "done")?;
    startup_debug_log("command:settings_complete_initial_setup:settings_saved");

    let mut archive_names = PersonaLoader::list_personas();
    archive_names.sort();
    let total_personas = archive_names.len();
    startup_debug_log("command:settings_complete_initial_setup:archive_listed");

    let persona_service = PersonaService::new(&conn);
    for (index, name) in archive_names.iter().enumerate() {
        if PersonaRepository::get_persona(&conn, name)
            .map_err(|e| SettingsError::database(&language, &e.to_string()))?
            .is_none()
        {
            persona_service
                .load_and_save_preset(name, &language)
                .map_err(|e| SettingsError::database(&language, &e.to_string()))?;
        }

        let _ = app_handle.emit(
            "setup_progress",
            SetupProgress {
                stage: "personas".to_string(),
                current: index + 1,
                total: total_personas,
            },
        );
    }

    let all_personas = PersonaRepository::list_personas(&conn)
        .map_err(|e| SettingsError::database(&language, &e.to_string()))?;
    startup_debug_log("command:settings_complete_initial_setup:personas_saved");
    for (index, persona) in all_personas.iter().enumerate() {
        let _ = persona_service.get_assembled_system_prompt(&persona.id, &language);
        let _ = app_handle.emit(
            "setup_progress",
            SetupProgress {
                stage: "caching".to_string(),
                current: index + 1,
                total: all_personas.len(),
            },
        );
    }

    let _ = app_handle.emit(
        "setup_progress",
        SetupProgress {
            stage: "model".to_string(),
            current: 0,
            total: 1,
        },
    );

    {
        startup_debug_log("command:settings_complete_initial_setup:model_load:block_start");
        let mut engine_lock = llm_state
            .inner()
            .0
            .lock()
            .map_err(|e| SettingsError::io(&language, &e.to_string()))?;
        startup_debug_log("command:settings_complete_initial_setup:model_load:llm_locked");

        if settings.get_inference_mode() == "local" && engine_lock.is_none() {
            let app_root = app_handle
                .path()
                .resource_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
            let adapters_dir = training_state
                .inner()
                .0
                .lock()
                .map_err(|e| SettingsError::io(&language, &e.to_string()))?
                .clone();
            let hardware = HardwareDetector::detect();
            let profile = HardwareDetector::inference_profile_for(
                PerformanceTier::from_str(&tier),
                hardware.physical_core_count,
            );

            let active_model = settings.get_active_model();
            if let Ok(handle) = LlmService::load_engine(
                &app_root,
                adapters_dir,
                profile,
                &active_model,
                &language,
                cache_state.inner().0.clone(),
            ) {
                *engine_lock = Some(handle);
            }
        }
        startup_debug_log("command:settings_complete_initial_setup:model_load:block_done");
    }

    let _ = app_handle.emit(
        "setup_progress",
        SetupProgress {
            stage: "done".to_string(),
            current: 1,
            total: 1,
        },
    );

    let result = SettingsService::get_settings(&settings, &api_keys);
    startup_debug_log("command:settings_complete_initial_setup:done");
    Ok(result)
}
