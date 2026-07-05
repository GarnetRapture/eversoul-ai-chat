use super::types::{AppSettings, ResetSummary, SettingsError};
use crate::infrastructure::database::DatabaseManager;
use crate::infrastructure::settings::SettingsManager;
use rusqlite::Connection;

pub struct SettingsService;

impl SettingsService {
    pub fn get_settings(settings: &SettingsManager) -> AppSettings {
        AppSettings {
            default_persona_id: settings.get_default_persona_id(),
            active_style_id: settings.get_active_style_id(),
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
