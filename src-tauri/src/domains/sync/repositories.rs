use rusqlite::{params, Connection, Result};
use std::time::SystemTime;

pub struct SyncRepository;

impl SyncRepository {
    /// 지정된 키의 메타데이터 값을 설정하고 최종 갱신 일시를 기록한다.
    pub fn set_metadata(conn: &Connection, key: &str, value: &str) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        conn.execute(
            "INSERT OR REPLACE INTO sync_metadata (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, now],
        )?;
        Ok(())
    }

    /// 지정된 키의 메타데이터 값을 읽어온다.
    pub fn get_metadata(conn: &Connection, key: &str) -> Result<Option<String>> {
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
