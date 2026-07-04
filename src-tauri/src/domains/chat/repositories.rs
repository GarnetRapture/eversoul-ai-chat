use super::types::{ChatMessage, ChatRoom};
use rusqlite::{params, Connection, Result};

pub struct ChatRepository;

impl ChatRepository {
    /// 대화방을 생성하여 데이터베이스에 추가한다.
    pub fn create_room(conn: &Connection, room: &ChatRoom) -> Result<()> {
        conn.execute(
            "INSERT INTO chat_room (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![room.id, room.title, room.created_at, room.updated_at],
        )?;
        Ok(())
    }

    /// 대화방 리스트를 전체 조회한다.
    pub fn list_rooms(conn: &Connection) -> Result<Vec<ChatRoom>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at FROM chat_room ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ChatRoom {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            if let Ok(r) = item {
                list.push(r);
            }
        }
        Ok(list)
    }

    /// 특정 대화방 내 최근 메시지들을 로드한다.
    pub fn list_messages(conn: &Connection, room_id: &str) -> Result<Vec<ChatMessage>> {
        let mut stmt = conn.prepare(
            "SELECT id, room_id, role, content, created_at FROM chat_message WHERE room_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![room_id], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                room_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            if let Ok(m) = item {
                list.push(m);
            }
        }
        Ok(list)
    }

    /// 단일 메시지를 데이터베이스에 추가한다.
    pub fn insert_message(conn: &Connection, msg: &ChatMessage) -> Result<()> {
        conn.execute(
            "INSERT INTO chat_message (id, room_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![msg.id, msg.room_id, msg.role, msg.content, msg.created_at],
        )?;

        // 대화방 최종 갱신 일시 업데이트
        conn.execute(
            "UPDATE chat_room SET updated_at = ?1 WHERE id = ?2",
            params![msg.created_at, msg.room_id],
        )?;

        Ok(())
    }

    /// 대화방 삭제 처리
    pub fn delete_room(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM chat_room WHERE id = ?1", params![id])?;
        Ok(())
    }
}
