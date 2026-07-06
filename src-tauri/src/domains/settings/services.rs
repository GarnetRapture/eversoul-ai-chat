use super::types::{AppSettings, HardwareProfile, ResetSummary, SettingsError};
use crate::domains::persona::repositories::PersonaRepository;
use crate::domains::persona::services::PersonaService;
use crate::infrastructure::database::DatabaseManager;
use crate::infrastructure::hardware::HardwareDetector;
use crate::infrastructure::settings::SettingsManager;
use rusqlite::Connection;

pub struct SettingsService;

impl SettingsService {
    pub fn get_settings(settings: &SettingsManager) -> AppSettings {
        AppSettings {
            default_persona_id: settings.get_default_persona_id(),
            active_style_id: settings.get_active_style_id(),
            language: settings.get_language(),
            language_configured: settings.has_language(),
            performance_tier: settings.get_performance_tier(),
            performance_configured: settings.has_performance_tier(),
        }
    }

    pub fn set_performance_tier(
        settings: &SettingsManager,
        tier: &str,
    ) -> Result<AppSettings, SettingsError> {
        if !matches!(tier, "light" | "balanced" | "performance") {
            return Err(SettingsError::Validation(tier.to_string()));
        }
        settings
            .set_performance_tier(tier)
            .map_err(|e| SettingsError::Io(e.to_string()))?;
        Ok(Self::get_settings(settings))
    }

    pub fn detect_hardware() -> HardwareProfile {
        let info = HardwareDetector::detect();
        let recommended_tier = HardwareDetector::recommend_tier(&info);

        HardwareProfile {
            physical_core_count: info.physical_core_count,
            logical_core_count: info.logical_core_count,
            total_memory_mb: info.total_memory_mb,
            recommended_tier: recommended_tier.as_str().to_string(),
        }
    }

    pub fn set_language(
        conn: &Connection,
        settings: &SettingsManager,
        language: &str,
    ) -> Result<AppSettings, SettingsError> {
        let result = Self::set_language_without_warmup(settings, language)?;
        Self::warm_up_localized_prompts(conn, language);
        Ok(result)
    }

    pub fn set_language_without_warmup(
        settings: &SettingsManager,
        language: &str,
    ) -> Result<AppSettings, SettingsError> {
        if !matches!(language, "ko" | "en" | "zh_cn") {
            return Err(SettingsError::Validation(language.to_string()));
        }
        settings
            .set_language(language)
            .map_err(|e| SettingsError::Io(e.to_string()))?;

        Ok(Self::get_settings(settings))
    }

    fn warm_up_localized_prompts(conn: &Connection, language: &str) {
        let service = PersonaService::new(conn);
        match PersonaRepository::list_personas(conn) {
            Ok(all_personas) => {
                for persona in all_personas {
                    if let Err(err) = service.get_assembled_system_prompt(&persona.id, language) {
                        eprintln!(
                            "언어 전환 후 정령 프롬프트 사전 캐시 실패 ({}/{}): {}",
                            persona.id, language, err
                        );
                    }
                }
            }
            Err(err) => {
                eprintln!("언어 전환 사전 캐시 대상 조회 실패: {}", err);
            }
        }
    }

    pub fn reset_all(
        conn: &Connection,
        settings: &SettingsManager,
    ) -> Result<ResetSummary, SettingsError> {
        let counts = DatabaseManager::reset_data(conn)
            .map_err(|e| SettingsError::Database(e.to_string()))?;

        settings
            .reset()
            .map_err(|e| SettingsError::Io(e.to_string()))?;

        Ok(ResetSummary {
            cleared_chat_rooms: counts.chat_rooms,
            cleared_chat_messages: counts.chat_messages,
            cleared_personas: counts.personas,
            cleared_styles: counts.styles,
            cleared_knowledge_chunks: counts.knowledge_chunks,
            cleared_persona_memories: counts.persona_memories,
        })
    }
}
