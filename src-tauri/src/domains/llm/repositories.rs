use rusqlite::{params, Connection, Result};

pub struct LlmRepository;

impl LlmRepository {
    /// LLM 설정 정보를 SQLite 데이터베이스에 보관한다.
    pub fn save_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO sync_metadata (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    /// LLM 설정 정보를 데이터베이스에서 읽어온다.
    pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
        let mut stmt = conn.prepare("SELECT value FROM sync_metadata WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }
}
