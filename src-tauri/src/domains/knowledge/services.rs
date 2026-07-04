use super::repositories::KnowledgeRepository;
use super::types::KnowledgePayload;
use rusqlite::Connection;

pub struct KnowledgeService<'a> {
    conn: &'a Connection,
}

impl<'a> KnowledgeService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    const DEFAULT_SEARCH_LIMIT: usize = 5;

    pub fn query_knowledge(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<Vec<KnowledgePayload>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let limit = limit.unwrap_or(Self::DEFAULT_SEARCH_LIMIT);
        KnowledgeRepository::search_chunks(self.conn, query, limit).map_err(|e| e.to_string())
    }
}
