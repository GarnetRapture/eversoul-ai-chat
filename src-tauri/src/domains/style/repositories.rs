use super::types::StyleProfile;
use rusqlite::{params, Connection, Result, Row};

pub struct StyleRepository;

impl StyleRepository {

    pub fn save_style(conn: &Connection, style: &StyleProfile) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO style_profile (
                id, name, tone, formality, emoji_usage, speech_rules, example_phrases, raw_json, is_active, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                style.id,
                style.name,
                style.tone,
                style.formality,
                style.emoji_usage as i64,
                style.speech_rules,
                style.example_phrases,
                style.raw_json,
                style.is_active as i64,
                style.created_at
            ],
        )?;
        Ok(())
    }

    pub fn get_style(conn: &Connection, id: &str) -> Result<Option<StyleProfile>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, tone, formality, emoji_usage, speech_rules, example_phrases, raw_json, is_active, created_at
             FROM style_profile WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_style(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_styles(conn: &Connection) -> Result<Vec<StyleProfile>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, tone, formality, emoji_usage, speech_rules, example_phrases, raw_json, is_active, created_at
             FROM style_profile",
        )?;
        let rows = stmt.query_map([], |row| Self::row_to_style(row))?;

        let mut list = Vec::new();
        for item in rows {
            if let Ok(s) = item {
                list.push(s);
            }
        }
        Ok(list)
    }

    fn row_to_style(row: &Row) -> Result<StyleProfile> {
        Ok(StyleProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            tone: row.get(2)?,
            formality: row.get(3)?,
            emoji_usage: row.get::<_, i64>(4)? != 0,
            speech_rules: row.get(5)?,
            example_phrases: row.get(6)?,
            raw_json: row.get(7)?,
            is_active: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
        })
    }
}
