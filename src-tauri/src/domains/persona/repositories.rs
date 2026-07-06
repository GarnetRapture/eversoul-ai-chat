use super::types::{PersonaConfig, PersonaLocalizedPrompt};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub struct PersonaRepository;

impl PersonaRepository {
    pub fn save_persona(conn: &Connection, config: &PersonaConfig) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO persona_profile (
                id, name, name_en, grade, race, class, sub_class, system_prompt, greeting, raw_json, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                config.id,
                config.name,
                config.name_en,
                config.grade,
                config.race,
                config.class,
                config.sub_class,
                config.system_prompt,
                config.greeting,
                config.raw_json,
                config.created_at
            ],
        )?;
        Ok(())
    }

    pub fn get_persona(conn: &Connection, id: &str) -> Result<Option<PersonaConfig>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, name_en, grade, race, class, sub_class, system_prompt, greeting, raw_json, created_at
             FROM persona_profile WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(PersonaConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                name_en: row.get(2)?,
                grade: row.get(3)?,
                race: row.get(4)?,
                class: row.get(5)?,
                sub_class: row.get(6)?,
                system_prompt: row.get(7)?,
                greeting: row.get(8)?,
                raw_json: row.get(9)?,
                created_at: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_personas(conn: &Connection) -> Result<Vec<PersonaConfig>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, name_en, grade, race, class, sub_class, system_prompt, greeting, raw_json, created_at
             FROM persona_profile"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PersonaConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                name_en: row.get(2)?,
                grade: row.get(3)?,
                race: row.get(4)?,
                class: row.get(5)?,
                sub_class: row.get(6)?,
                system_prompt: row.get(7)?,
                greeting: row.get(8)?,
                raw_json: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            if let Ok(p) = item {
                list.push(p);
            }
        }
        Ok(list)
    }

    pub fn get_localized_prompt(
        conn: &Connection,
        persona_id: &str,
        language: &str,
        source_updated_at: &str,
    ) -> Result<Option<PersonaLocalizedPrompt>> {
        conn.query_row(
            "SELECT persona_id, language, localized_name, assembled_prompt, source_updated_at, cached_at
             FROM persona_localized_prompt
             WHERE persona_id = ?1 AND language = ?2 AND source_updated_at = ?3",
            params![persona_id, language, source_updated_at],
            |row| {
                Ok(PersonaLocalizedPrompt {
                    persona_id: row.get(0)?,
                    language: row.get(1)?,
                    localized_name: row.get(2)?,
                    assembled_prompt: row.get(3)?,
                    source_updated_at: row.get(4)?,
                    cached_at: row.get(5)?,
                })
            },
        )
        .optional()
    }

    pub fn save_localized_prompt(conn: &Connection, entry: &PersonaLocalizedPrompt) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO persona_localized_prompt (
                persona_id, language, localized_name, assembled_prompt, source_updated_at, cached_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.persona_id,
                entry.language,
                entry.localized_name,
                entry.assembled_prompt,
                entry.source_updated_at,
                entry.cached_at
            ],
        )?;
        Ok(())
    }

    pub fn count_localized_prompts_for_language(conn: &Connection, language: &str) -> Result<i64> {
        conn.query_row(
            "SELECT COUNT(*) FROM persona_localized_prompt WHERE language = ?1",
            params![language],
            |row| row.get(0),
        )
    }
}
