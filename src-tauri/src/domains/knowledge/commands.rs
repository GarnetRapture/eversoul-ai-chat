use tauri::State;
use crate::domains::auth::commands::DbState;
use super::types::KnowledgePayload;
use super::services::KnowledgeService;

#[tauri::command]
pub fn knowledge_search(
    db_state: State<'_, DbState>,
    query: String,
    limit: usize,
) -> Result<Vec<KnowledgePayload>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let service = KnowledgeService::new(&conn);
    service.query_knowledge(&query, limit)
}
