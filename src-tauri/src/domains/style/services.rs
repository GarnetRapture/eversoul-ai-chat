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

    /// 사용 가능한 모든 스타일 프로필 목록을 획득한다.
    pub fn get_available_styles(&self) -> Result<Vec<StyleProfile>, StyleError> {
        StyleRepository::list_styles(self.conn).map_err(|e| StyleError::Database(e.to_string()))
    }

    /// 스타일 프로필의 설정값을 수정하여 로컬 SQLite에 반영한다.
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

    /// 지정된 스타일을 현재 활성 스타일로 선택한다.
    pub fn select_active_style(&self, id: &str) -> Result<StyleProfile, StyleError> {
        let existing = StyleRepository::get_style(self.conn, id)
            .map_err(|e| StyleError::Database(e.to_string()))?
            .ok_or_else(|| StyleError::NotFound(id.to_string()))?;

        StyleRepository::set_active_style(self.conn, id)
            .map_err(|e| StyleError::Database(e.to_string()))?;

        Ok(StyleProfile {
            is_active: true,
            ..existing
        })
    }

    /// 현재 활성화된 스타일 프로필을 조회한다.
    pub fn get_active_style(&self) -> Result<Option<StyleProfile>, StyleError> {
        StyleRepository::get_active_style(self.conn)
            .map_err(|e| StyleError::Database(e.to_string()))
    }

    /// 활성 스타일을 시스템 프롬프트에 삽입할 문체 지침 문자열로 변환한다.
    ///
    /// 활성 스타일이 없으면 빈 문자열을 반환하여 기존 페르소나 어투를 그대로 유지한다.
    pub fn get_assembled_style_prompt(&self) -> Result<String, StyleError> {
        let active = self.get_active_style()?;

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
