use rusqlite::{params, Connection, Result};
use super::types::UserSession;

pub struct AuthRepository;

impl AuthRepository {
    /// 데이터베이스에 현재 세션을 저장한다. 기존 세션은 삭제한다.
    pub fn save_session(conn: &Connection, session: &UserSession) -> Result<()> {
        conn.execute("DELETE FROM auth_session", [])?;
        conn.execute(
            "INSERT INTO auth_session (token, email, username, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![session.token, session.email, session.username, session.created_at],
        )?;
        Ok(())
    }

    /// 현재 저장된 활성 세션을 가져온다.
    pub fn get_session(conn: &Connection) -> Result<Option<UserSession>> {
        let mut stmt = conn.prepare("SELECT token, email, username, created_at FROM auth_session LIMIT 1")?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(UserSession {
                token: row.get(0)?,
                email: row.get(1)?,
                username: row.get(2)?,
                created_at: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 저장된 세션을 파기(로그아웃)한다.
    pub fn clear_session(conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM auth_session", [])?;
        Ok(())
    }
}
