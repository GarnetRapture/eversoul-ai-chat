use rusqlite::Connection;
use super::types::KnowledgePayload;
use super::repositories::KnowledgeRepository;

pub struct KnowledgeService<'a> {
    conn: &'a Connection,
}

impl<'a> KnowledgeService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 로컬 지식 데이터베이스에서 질의어에 상응하는 지식 청크들을 발췌한다.
    pub fn query_knowledge(&self, query: &str, limit: usize) -> Result<Vec<KnowledgePayload>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        KnowledgeRepository::search_chunks(self.conn, query, limit).map_err(|e| e.to_string())
    }
}
