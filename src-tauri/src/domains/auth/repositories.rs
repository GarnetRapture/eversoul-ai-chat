use super::types::UserSession;
use rusqlite::{params, Connection, Result};

pub struct AuthRepository;

impl AuthRepository {
    pub fn save_session(conn: &Connection, session: &UserSession) -> Result<()> {
        conn.execute("DELETE FROM auth_session", [])?;
        conn.execute(
            "INSERT INTO auth_session (token, email, username, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                session.token,
                session.email,
                session.username,
                session.created_at
            ],
        )?;
        Ok(())
    }

    pub fn get_session(conn: &Connection) -> Result<Option<UserSession>> {
        let mut stmt =
            conn.prepare("SELECT token, email, username, created_at FROM auth_session LIMIT 1")?;
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

    pub fn clear_session(conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM auth_session", [])?;
        Ok(())
    }
}
