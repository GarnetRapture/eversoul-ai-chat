use super::memory::{cosine_similarity, deserialize_vector, serialize_vector};
use super::types::{ChatMessage, ChatRoom};
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

pub struct ChatRepository;

impl ChatRepository {
    pub fn create_room(conn: &Connection, room: &ChatRoom) -> Result<()> {
        conn.execute(
            "INSERT INTO chat_room (id, title, persona_id, session_started_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                room.id,
                room.title,
                room.persona_id.as_deref(),
                room.session_started_at,
                room.created_at,
                room.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn list_rooms(conn: &Connection) -> Result<Vec<ChatRoom>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, persona_id, session_started_at, created_at, updated_at FROM chat_room ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ChatRoom {
                id: row.get(0)?,
                title: row.get(1)?,
                persona_id: row.get(2)?,
                session_started_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
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

    pub fn find_latest_room_by_persona(
        conn: &Connection,
        persona_id: &str,
    ) -> Result<Option<ChatRoom>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, persona_id, session_started_at, created_at, updated_at FROM chat_room WHERE persona_id = ?1 ORDER BY updated_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![persona_id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(ChatRoom {
                id: row.get(0)?,
                title: row.get(1)?,
                persona_id: row.get(2)?,
                session_started_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            }));
        }

        Ok(None)
    }

    pub fn list_rooms_by_persona(conn: &Connection, persona_id: &str) -> Result<Vec<ChatRoom>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, persona_id, session_started_at, created_at, updated_at FROM chat_room WHERE persona_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![persona_id], |row| {
            Ok(ChatRoom {
                id: row.get(0)?,
                title: row.get(1)?,
                persona_id: row.get(2)?,
                session_started_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
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

    pub fn list_recent_messages(
        conn: &Connection,
        room_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>> {
        let mut stmt = conn.prepare(
            "SELECT id, room_id, role, content, created_at FROM chat_message
             WHERE room_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![room_id, limit as i64], |row| {
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
            if let Ok(message) = item {
                list.push(message);
            }
        }
        list.reverse();
        Ok(list)
    }

    pub fn insert_message(conn: &Connection, msg: &ChatMessage) -> Result<()> {
        conn.execute(
            "INSERT INTO chat_message (id, room_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![msg.id, msg.room_id, msg.role, msg.content, msg.created_at],
        )?;

        conn.execute(
            "UPDATE chat_room SET updated_at = ?1 WHERE id = ?2",
            params![msg.created_at, msg.room_id],
        )?;

        Ok(())
    }

    pub fn delete_room(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM chat_room WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn insert_episodic_memory(
        conn: &Connection,
        id: &str,
        persona_id: &str,
        memory_text: &str,
        memory_vector: &[f32],
        created_at: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO persona_memory (id, persona_id, memory_type, memory_text, memory_vector, created_at)
             VALUES (?1, ?2, 'episodic', ?3, ?4, ?5)",
            params![
                id,
                persona_id,
                memory_text,
                serialize_vector(memory_vector),
                created_at
            ],
        )?;
        Ok(())
    }

    pub fn count_episodic_memories(conn: &Connection, persona_id: &str) -> Result<usize> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM persona_memory WHERE persona_id = ?1 AND memory_type = 'episodic'",
            params![persona_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn list_episodic_memories(
        conn: &Connection,
        persona_id: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT memory_text FROM persona_memory
             WHERE persona_id = ?1 AND memory_type = 'episodic'
             ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![persona_id, limit as i64], |row| {
            row.get::<_, String>(0)
        })?;

        let mut list = Vec::new();
        for item in rows {
            if let Ok(text) = item {
                list.push(text);
            }
        }
        Ok(list)
    }

    pub fn search_episodic_memories(
        conn: &Connection,
        persona_id: &str,
        query_vector: &[f32],
        limit: usize,
        candidate_limit: usize,
    ) -> Result<Vec<String>> {
        if query_vector.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare(
            "SELECT memory_text, memory_vector FROM persona_memory
             WHERE persona_id = ?1 AND memory_type = 'episodic' AND length(memory_vector) > 0
             ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![persona_id, candidate_limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;

        let mut scored = Vec::new();
        for item in rows {
            if let Ok((memory_text, vector_bytes)) = item {
                if let Some(memory_vector) = deserialize_vector(&vector_bytes) {
                    if let Some(score) = cosine_similarity(query_vector, &memory_vector) {
                        scored.push((score, memory_text));
                    }
                }
            }
        }

        scored.sort_by(|left, right| {
            right
                .0
                .partial_cmp(&left.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(scored
            .into_iter()
            .take(limit)
            .map(|(_, memory_text)| memory_text)
            .collect())
    }

    pub fn upsert_semantic_memory(
        conn: &Connection,
        persona_id: &str,
        memory_text: &str,
        memory_vector: &[f32],
        created_at: &str,
    ) -> Result<()> {
        let id = format!("semantic-{}", persona_id);
        conn.execute(
            "INSERT OR REPLACE INTO persona_memory (id, persona_id, memory_type, memory_text, memory_vector, created_at)
             VALUES (?1, ?2, 'semantic', ?3, ?4, ?5)",
            params![
                id,
                persona_id,
                memory_text,
                serialize_vector(memory_vector),
                created_at
            ],
        )?;
        Ok(())
    }

    pub fn get_semantic_memory(conn: &Connection, persona_id: &str) -> Result<Option<String>> {
        let mut stmt = conn.prepare(
            "SELECT memory_text FROM persona_memory WHERE persona_id = ?1 AND memory_type = 'semantic' LIMIT 1",
        )?;
        let mut rows = stmt.query(params![persona_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn count_messages_by_persona(conn: &Connection) -> Result<HashMap<String, usize>> {
        let mut stmt = conn.prepare(
            "SELECT cr.persona_id, COUNT(cm.id)
             FROM chat_room cr
             JOIN chat_message cm ON cm.room_id = cr.id
             WHERE cr.persona_id IS NOT NULL
             GROUP BY cr.persona_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;

        let mut map = HashMap::new();
        for item in rows {
            if let Ok((persona_id, count)) = item {
                map.insert(persona_id, count);
            }
        }
        Ok(map)
    }

    pub fn count_episodic_memories_by_persona(conn: &Connection) -> Result<HashMap<String, usize>> {
        let mut stmt = conn.prepare(
            "SELECT persona_id, COUNT(*) FROM persona_memory
             WHERE memory_type = 'episodic'
             GROUP BY persona_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;

        let mut map = HashMap::new();
        for item in rows {
            if let Ok((persona_id, count)) = item {
                map.insert(persona_id, count);
            }
        }
        Ok(map)
    }
}
