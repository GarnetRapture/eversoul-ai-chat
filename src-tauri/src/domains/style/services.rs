use super::repositories::StyleRepository;
use super::types::{StyleError, StyleProfile, UpdateStyleRequest};
use rusqlite::Connection;

pub struct StyleService<'a> {
    conn: &'a Connection,
}

impl<'a> StyleService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_available_styles(
        &self,
        active_style_id: Option<&str>,
    ) -> Result<Vec<StyleProfile>, StyleError> {
        let styles = StyleRepository::list_styles(self.conn)
            .map_err(|e| StyleError::Database(e.to_string()))?;

        Ok(styles
            .into_iter()
            .map(|mut style| {
                style.is_active = active_style_id == Some(style.id.as_str());
                style
            })
            .collect())
    }

    pub fn update_style_settings(&self, req: UpdateStyleRequest) -> Result<(), StyleError> {
        let existing = StyleRepository::get_style(self.conn, &req.id)
            .map_err(|e| StyleError::Database(e.to_string()))?;

        if let Some(mut style) = existing {
            style.tone = req.tone;
            style.formality = req.formality;
            style.emoji_usage = req.emoji_usage;
            style.speech_rules = req.speech_rules;
            StyleRepository::save_style(self.conn, &style)
                .map_err(|e| StyleError::Database(e.to_string()))
        } else {
            Err(StyleError::NotFound(req.id))
        }
    }

    pub fn get_style_for_activation(&self, id: &str) -> Result<StyleProfile, StyleError> {
        StyleRepository::get_style(self.conn, id)
            .map_err(|e| StyleError::Database(e.to_string()))?
            .ok_or_else(|| StyleError::NotFound(id.to_string()))
    }

    pub fn get_style_by_id(&self, id: &str) -> Result<Option<StyleProfile>, StyleError> {
        StyleRepository::get_style(self.conn, id).map_err(|e| StyleError::Database(e.to_string()))
    }

    pub fn get_assembled_style_prompt(
        &self,
        active_style_id: Option<&str>,
    ) -> Result<String, StyleError> {
        let active = match active_style_id {
            Some(id) => self.get_style_by_id(id)?,
            None => None,
        };

        Ok(match active {
            Some(style) => format!(
                "\n[말투 스타일 지침]\n- 어조: {}\n- 격식: {}\n- 이모티콘 사용: {}\n- 세부 규칙: {}\n",
                style.tone,
                style.formality,
                if style.emoji_usage { "허용" } else { "비허용" },
                style.speech_rules
            ),
            None => String::new(),
        })
    }
}
