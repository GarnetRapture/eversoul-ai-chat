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

    /// 지식 검색 결과 개수 제한 기본값 (호출 측에서 별도로 지정하지 않을 경우 적용).
    const DEFAULT_SEARCH_LIMIT: usize = 5;

    /// 로컬 지식 데이터베이스에서 질의어에 상응하는 지식 청크들을 발췌한다.
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
