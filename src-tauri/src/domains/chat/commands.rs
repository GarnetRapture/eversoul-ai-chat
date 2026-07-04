use tauri::State;
use crate::domains::auth::commands::DbState;
use crate::domains::llm::commands::LlmState;
use super::types::{ChatRoom, ChatMessage, SendMessageRequest, ChatError};
use super::services::ChatService;

#[tauri::command]
pub fn chat_create_room(
    db_state: State<'_, DbState>,
    title: String,
) -> Result<ChatRoom, ChatError> {
    let conn = db_state.0.lock().map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    service.create_chat_room(&title)
}

#[tauri::command]
pub fn chat_list_rooms(
    db_state: State<'_, DbState>,
) -> Result<Vec<ChatRoom>, ChatError> {
    let conn = db_state.0.lock().map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    service.get_chat_rooms()
}

#[tauri::command]
pub fn chat_list_messages(
    db_state: State<'_, DbState>,
    room_id: String,
) -> Result<Vec<ChatMessage>, ChatError> {
    let conn = db_state.0.lock().map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    service.get_room_messages(&room_id)
}

#[tauri::command]
pub async fn chat_send_message(
    db_state: State<'_, DbState>,
    llm_state: State<'_, LlmState>,
    room_id: String,
    content: String,
    persona_id: String,
) -> Result<ChatMessage, ChatError> {
    // 1. Connection 획득 (Mutex 락 획득 범위 관리)
    let conn = db_state.0.lock().map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    let req = SendMessageRequest { room_id, content, persona_id };

    // 2. LLM State 획득
    let engine = llm_state.0.lock().map_err(|e| ChatError::Unknown(e.to_string()))?;

    // 3. 메시지 프로세싱 및 LLM 추론 위임 호출 (Qwen 챗 템플릿 준수)
    let res = service.process_message(req, |system_prompt, history| async move {
        if let Some(ref engine_instance) = *engine {
            let mut full_prompt = String::new();
            
            // Qwen 2.5 챗 템플릿 포맷 조립
            // 시스템 가이드 (Persona 및 RAG 컨텍스트)
            full_prompt.push_str(&format!("<|im_start|>system\n{}<|im_end|>\n", system_prompt));
            
            // 최근 10개 히스토리만 발췌
            for msg in history.iter().take(10) {
                full_prompt.push_str(&format!("<|im_start|>{}\n{}<|im_end|>\n", msg.role, msg.content));
            }
            
            // 최종 AI 응답 트리거
            full_prompt.push_str("<|im_start|>assistant\n");

            // 로컬 CPU GGUF 추론 실행
            let infer_res = engine_instance.infer(&full_prompt, Some(256))
                .map_err(|e| ChatError::LlmInferenceFailed(e.to_string()))?;
                
            Ok(infer_res)
        } else {
            Err(ChatError::LlmEngineNotLoaded)
        }
    }).await;

    res
}
