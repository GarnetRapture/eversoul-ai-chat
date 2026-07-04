use super::services::KnowledgeService;
use super::types::KnowledgePayload;
use crate::domains::auth::commands::DbState;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn knowledge_search(
    db_state: State<'_, DbState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<KnowledgePayload>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let service = KnowledgeService::new(&conn);
    service.query_knowledge(&query, limit)
}
