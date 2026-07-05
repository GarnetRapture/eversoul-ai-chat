use super::types::KnowledgePayload;
use rusqlite::{params, Connection, Result};

pub struct KnowledgeRepository;

impl KnowledgeRepository {
    pub fn insert_chunk(conn: &Connection, payload: &KnowledgePayload) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO knowledge_chunk (id, document_name, chunk_text, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                payload.id,
                payload.document_name,
                payload.chunk_text,
                payload.created_at
            ],
        )?;
        Ok(())
    }

    pub fn search_chunks(
        conn: &Connection,
        query: &str,
        limit: usize,
    ) -> Result<Vec<KnowledgePayload>> {
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, document_name, chunk_text, created_at FROM knowledge_chunk
             WHERE chunk_text LIKE ?1 LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok(KnowledgePayload {
                id: row.get(0)?,
                document_name: row.get(1)?,
                chunk_text: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            if let Ok(c) = item {
                list.push(c);
            }
        }
        Ok(list)
    }
}
