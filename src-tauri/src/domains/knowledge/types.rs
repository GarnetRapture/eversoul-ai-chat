use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePayload {
    pub id: String,
    pub document_name: String,
    pub chunk_text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSearchQuery {
    pub query: String,
    pub limit: usize,
}
