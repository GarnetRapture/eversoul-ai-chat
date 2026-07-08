use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct DatabaseManager {
    path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ResetCounts {
    pub chat_rooms: usize,
    pub chat_messages: usize,
    pub personas: usize,
    pub styles: usize,
    pub knowledge_chunks: usize,
    pub persona_memories: usize,
}

impl DatabaseManager {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn connect(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path)?;
        Self::migrate(&conn)?;
        Ok(conn)
    }

    pub fn reset_data(conn: &Connection) -> Result<ResetCounts> {
        let counts = ResetCounts {
            chat_rooms: count_rows(conn, "chat_room")?,
            chat_messages: count_rows(conn, "chat_message")?,
            personas: count_rows(conn, "persona_profile")?,
            styles: count_rows(conn, "style_profile")?,
            knowledge_chunks: count_rows(conn, "knowledge_chunk")?,
            persona_memories: count_rows(conn, "persona_memory")?,
        };

        conn.execute_batch(
            "
            DELETE FROM chat_message;
            DELETE FROM chat_room;
            DELETE FROM persona_memory;
            DELETE FROM persona_localized_prompt;
            DELETE FROM persona_profile;
            DELETE FROM style_profile;
            DELETE FROM knowledge_chunk;
            DELETE FROM auth_session;
            DELETE FROM sync_metadata;
            ",
        )?;

        Ok(counts)
    }

    fn migrate(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS auth_session (
                token TEXT NOT NULL,
                email TEXT NOT NULL,
                username TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS chat_room (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                persona_id TEXT,
                session_started_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS chat_message (
                id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL,
                persona_id TEXT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (room_id) REFERENCES chat_room(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS persona_profile (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                name_en TEXT,
                grade TEXT,
                race TEXT,
                class TEXT,
                sub_class TEXT,
                system_prompt TEXT NOT NULL,
                greeting TEXT,
                raw_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS persona_localized_prompt (
                persona_id TEXT NOT NULL,
                language TEXT NOT NULL,
                localized_name TEXT NOT NULL,
                assembled_prompt TEXT NOT NULL,
                source_updated_at TEXT NOT NULL,
                cached_at TEXT NOT NULL,
                PRIMARY KEY (persona_id, language, source_updated_at)
            );

            CREATE TABLE IF NOT EXISTS persona_memory (
                id TEXT PRIMARY KEY,
                persona_id TEXT NOT NULL,
                memory_type TEXT NOT NULL,
                memory_text TEXT NOT NULL,
                memory_vector BLOB NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS style_profile (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                tone TEXT NOT NULL,
                formality TEXT NOT NULL,
                emoji_usage INTEGER NOT NULL,
                speech_rules TEXT NOT NULL,
                example_phrases TEXT NOT NULL,
                raw_json TEXT NOT NULL,
                is_active INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS knowledge_chunk (
                id TEXT PRIMARY KEY,
                document_name TEXT NOT NULL,
                chunk_text TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_chat_room_persona_id ON chat_room(persona_id);
            CREATE INDEX IF NOT EXISTS idx_chat_message_room_id ON chat_message(room_id);
            CREATE INDEX IF NOT EXISTS idx_chat_message_persona_id ON chat_message(persona_id);
            CREATE INDEX IF NOT EXISTS idx_persona_memory_persona_id ON persona_memory(persona_id);
            CREATE INDEX IF NOT EXISTS idx_knowledge_chunk_text ON knowledge_chunk(chunk_text);
            ",
        )
    }
}

fn count_rows(conn: &Connection, table_name: &str) -> Result<usize> {
    let sql = format!("SELECT COUNT(*) FROM {table_name}");
    let count: i64 = conn.query_row(&sql, [], |row| row.get(0))?;
    Ok(count as usize)
}
