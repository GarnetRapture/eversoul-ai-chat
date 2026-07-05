use super::types::PersonaConfig;
use rusqlite::{params, Connection, Result};

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
}
