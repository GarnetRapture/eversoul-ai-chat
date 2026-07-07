use super::services::SettingsService;
use super::types::{AppSettings, HardwareProfile, ResetSummary, SettingsError, SetupProgress};
use crate::domains::auth::commands::DbState;
use crate::domains::llm::commands::LlmState;
use crate::domains::llm::services::LlmService;
use crate::domains::persona::repositories::PersonaRepository;
use crate::domains::persona::services::PersonaService;
use crate::domains::training::commands::TrainingState;
use crate::infrastructure::compress::PersonaLoader;
use crate::infrastructure::hardware::{HardwareDetector, PerformanceTier};
use crate::infrastructure::settings::SettingsManager;
use crate::startup_debug_log;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct SettingsState(pub Mutex<SettingsManager>);

#[tauri::command(rename_all = "snake_case")]
pub fn settings_get(
    settings_state: State<'_, SettingsState>,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_get:start");
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    let result = SettingsService::get_settings(&settings);
    startup_debug_log("command:settings_get:done");
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_reset(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
) -> Result<ResetSummary, SettingsError> {
    startup_debug_log("command:settings_reset:start");
    let conn = db_state
        .0
        .lock()
        .map_err(|e| SettingsError::Database(e.to_string()))?;
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    let result = SettingsService::reset_all(&conn, &settings);
    startup_debug_log("command:settings_reset:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_language(
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    language: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_language:start");
    let conn = db_state
        .0
        .lock()
        .map_err(|e| SettingsError::Database(e.to_string()))?;
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    let result = SettingsService::set_language(&conn, &settings, &language);
    startup_debug_log("command:settings_set_language:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_performance_tier(
    settings_state: State<'_, SettingsState>,
    tier: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_performance_tier:start");
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    let result = SettingsService::set_performance_tier(&settings, &tier);
    startup_debug_log("command:settings_set_performance_tier:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set_setup_stage(
    settings_state: State<'_, SettingsState>,
    stage: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_setup_stage:start");
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    let result = SettingsService::set_setup_stage(&settings, &stage);
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
    show_reasoning: bool,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_set_show_reasoning:start");
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    let result = SettingsService::set_show_reasoning(&settings, show_reasoning);
    startup_debug_log("command:settings_set_show_reasoning:done");
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_complete_initial_setup(
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    settings_state: State<'_, SettingsState>,
    llm_state: State<'_, LlmState>,
    training_state: State<'_, TrainingState>,
    language: String,
    tier: String,
) -> Result<AppSettings, SettingsError> {
    startup_debug_log("command:settings_complete_initial_setup:start");
    let conn = db_state
        .0
        .lock()
        .map_err(|e| SettingsError::Database(e.to_string()))?;
    startup_debug_log("command:settings_complete_initial_setup:db_locked");
    let settings = settings_state
        .0
        .lock()
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    startup_debug_log("command:settings_complete_initial_setup:settings_locked");

    SettingsService::set_language_without_warmup(&settings, &language)?;
    SettingsService::set_performance_tier(&settings, &tier)?;
    SettingsService::set_setup_stage(&settings, "done")?;
    startup_debug_log("command:settings_complete_initial_setup:settings_saved");

    let mut archive_names = PersonaLoader::list_personas();
    archive_names.sort();
    let total_personas = archive_names.len();
    startup_debug_log("command:settings_complete_initial_setup:archive_listed");

    let persona_service = PersonaService::new(&conn);
    for (index, name) in archive_names.iter().enumerate() {
        if PersonaRepository::get_persona(&conn, name)
            .map_err(|e| SettingsError::Database(e.to_string()))?
            .is_none()
        {
            persona_service
                .load_and_save_preset(name)
                .map_err(|e| SettingsError::Database(e.to_string()))?;
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
        .map_err(|e| SettingsError::Database(e.to_string()))?;
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
            .map_err(|e| SettingsError::Io(e.to_string()))?;
        startup_debug_log("command:settings_complete_initial_setup:model_load:llm_locked");

        if engine_lock.is_none() {
            let app_root = app_handle
                .path()
                .resource_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
            let adapters_dir = training_state
                .inner()
                .0
                .lock()
                .map_err(|e| SettingsError::Io(e.to_string()))?
                .clone();
            let hardware = HardwareDetector::detect();
            let profile = HardwareDetector::inference_profile_for(
                PerformanceTier::from_str(&tier),
                hardware.physical_core_count,
            );

            let active_model = settings.get_active_model();
            if let Ok(handle) = LlmService::load_engine(&app_root, adapters_dir, profile, &active_model) {
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

    let result = SettingsService::get_settings(&settings);
    startup_debug_log("command:settings_complete_initial_setup:done");
    Ok(result)
}
