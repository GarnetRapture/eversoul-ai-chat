use super::types::KnowledgePayload;
use rusqlite::{params, Connection, Result};

pub struct KnowledgeRepository;

impl KnowledgeRepository {
    /// 지식 청크 데이터를 데이터베이스에 적재한다.
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

    /// 단순 LIKE 쿼리를 이용해 본문 텍스트 내 키워드를 검색한다.
    /// SQLite3 기본 내장 검색으로 대용량 형태소 분석기 없이 빠르고 견고하게 작동하도록 설계.
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
