use super::services::ChatService;
use super::types::{ChatError, ChatMessage, ChatRoom, SendMessageRequest};
use crate::domains::auth::commands::DbState;
use crate::domains::llm::commands::LlmState;
use crate::domains::settings::commands::SettingsState;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn chat_create_room(
    db_state: State<'_, DbState>,
    title: String,
) -> Result<ChatRoom, ChatError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    service.create_chat_room(&title)
}

#[tauri::command(rename_all = "snake_case")]
pub fn chat_create_session_room(
    db_state: State<'_, DbState>,
    title: String,
    persona_id: String,
) -> Result<ChatRoom, ChatError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    service.create_chat_session_room(&title, Some(persona_id))
}

#[tauri::command(rename_all = "snake_case")]
pub fn chat_list_rooms(db_state: State<'_, DbState>) -> Result<Vec<ChatRoom>, ChatError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    service.get_chat_rooms()
}

#[tauri::command(rename_all = "snake_case")]
pub fn chat_get_latest_session_room(
    db_state: State<'_, DbState>,
    persona_id: String,
) -> Result<Option<ChatRoom>, ChatError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    service.get_latest_session_room(&persona_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn chat_list_messages(
    db_state: State<'_, DbState>,
    room_id: String,
) -> Result<Vec<ChatMessage>, ChatError> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| ChatError::Database(e.to_string()))?;
    let service = ChatService::new(&conn);
    service.get_room_messages(&room_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn chat_send_message(
    db_state: State<'_, DbState>,
    llm_state: State<'_, LlmState>,
    settings_state: State<'_, SettingsState>,
    room_id: String,
    content: String,
    persona_id: String,
) -> Result<ChatMessage, ChatError> {
    let req = SendMessageRequest {
        room_id: room_id.clone(),
        content,
        persona_id,
    };

    let query_vector = {
        let engine_lock = llm_state
            .0
            .lock()
            .map_err(|e| ChatError::Unknown(e.to_string()))?;
        let engine_instance = engine_lock.as_ref().ok_or(ChatError::LlmEngineNotLoaded)?;
        engine_instance
            .embed_text(&req.content)
            .map_err(|e| ChatError::LlmInferenceFailed(e.to_string()))?
    };

    let (system_prompt, history) = {
        let conn = db_state
            .0
            .lock()
            .map_err(|e| ChatError::Database(e.to_string()))?;
        let settings = settings_state
            .0
            .lock()
            .map_err(|e| ChatError::Unknown(e.to_string()))?;
        let service = ChatService::new(&conn);
        service.prepare_message_context(&req, &settings, &query_vector)?
    };

    let engine_lock = llm_state
        .0
        .lock()
        .map_err(|e| ChatError::Unknown(e.to_string()))?;
    let engine_instance = engine_lock.as_ref().ok_or(ChatError::LlmEngineNotLoaded)?;

    let mut full_prompt = String::new();
    full_prompt.push_str(&format!(
        "<|im_start|>system\n{}<|im_end|>\n",
        system_prompt
    ));

    for msg in &history {
        full_prompt.push_str(&format!(
            "<|im_start|>{}\n{}<|im_end|>\n",
            msg.role, msg.content
        ));
    }

    full_prompt.push_str("<|im_start|>assistant\n");

    let ai_text = engine_instance
        .infer(&full_prompt, Some(256), Some(&req.persona_id))
        .map_err(|e| ChatError::LlmInferenceFailed(e.to_string()))?;
    drop(engine_lock);

    let ai_msg = {
        let conn = db_state
            .0
            .lock()
            .map_err(|e| ChatError::Database(e.to_string()))?;
        let service = ChatService::new(&conn);
        service.save_ai_response(&room_id, ai_text.clone())?
    };

    {
        let engine_lock = llm_state.0.lock();
        let db_lock = db_state.0.lock();
        if let (Ok(engine_lock), Ok(conn)) = (engine_lock, db_lock) {
            if let Some(ref engine_instance) = *engine_lock {
                let service = ChatService::new(&conn);
                if let Err(err) = service.accumulate_memory(
                    &req.persona_id,
                    &req.content,
                    &ai_text,
                    engine_instance,
                ) {
                    eprintln!("정령 누적 기억 처리 실패: {}", err);
                }
            }
        }
    }

    Ok(ai_msg)
}
