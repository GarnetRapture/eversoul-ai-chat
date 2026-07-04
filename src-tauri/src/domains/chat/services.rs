use rusqlite::Connection;
use std::time::SystemTime;
use uuid::Uuid;

use super::repositories::ChatRepository;
use super::types::{ChatError, ChatMessage, ChatRoom, SendMessageRequest};
use crate::domains::knowledge::services::KnowledgeService;
use crate::domains::persona::services::PersonaService;
use crate::domains::style::services::StyleService;

pub struct ChatService<'a> {
    conn: &'a Connection,
}

impl<'a> ChatService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 새 대화방을 생성한다.
    pub fn create_chat_room(&self, title: &str) -> Result<ChatRoom, ChatError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        let room = ChatRoom {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        ChatRepository::create_room(self.conn, &room)
            .map(|_| room)
            .map_err(|e| ChatError::Database(e.to_string()))
    }

    /// 대화방 목록을 조회한다.
    pub fn get_chat_rooms(&self) -> Result<Vec<ChatRoom>, ChatError> {
        ChatRepository::list_rooms(self.conn).map_err(|e| ChatError::Database(e.to_string()))
    }

    /// 대화방 ID에 따라 메시지 리스트를 로드한다.
    pub fn get_room_messages(&self, room_id: &str) -> Result<Vec<ChatMessage>, ChatError> {
        ChatRepository::list_messages(self.conn, room_id)
            .map_err(|e| ChatError::Database(e.to_string()))
    }

    /// 사용자 메시지를 기록하고, 페르소나 및 로컬 지식 RAG 지침을 조립하여 AI 응답을 수립한다.
    ///
    /// 로컬 CPU LLM 추론과 SQLite 접근은 전부 동기 작업이므로, `MutexGuard`(non-Send)를
    /// `.await` 경계 너머로 들고 있지 않도록 이 메서드 전체를 동기로 구성한다.
    pub fn process_message<F>(
        &self,
        req: SendMessageRequest,
        llm_infer_fn: F,
    ) -> Result<ChatMessage, ChatError>
    where
        F: FnOnce(String, Vec<ChatMessage>) -> Result<String, ChatError>,
    {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        // 1. 사용자 메시지 생성 및 SQLite 저장
        let user_msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            room_id: req.room_id.clone(),
            role: "user".to_string(),
            content: req.content.clone(),
            created_at: now.clone(),
        };
        ChatRepository::insert_message(self.conn, &user_msg)
            .map_err(|e| ChatError::Database(e.to_string()))?;

        // 2. 페르소나 시스템 프롬프트 조립
        let persona_service = PersonaService::new(self.conn);
        let mut system_prompt = persona_service
            .get_assembled_system_prompt(&req.persona_id)
            .map_err(|e| ChatError::Database(e))?;

        // 3. 로컬 지식 기반 유사 검색(RAG)
        let knowledge_service = KnowledgeService::new(self.conn);
        if let Ok(chunks) = knowledge_service.query_knowledge(&req.content, Some(2)) {
            if !chunks.is_empty() {
                let mut knowledge_context = String::from("\n[참고 지식 데이터]\n");
                for (i, chunk) in chunks.iter().enumerate() {
                    knowledge_context.push_str(&format!("{}. {}\n", i + 1, chunk.chunk_text));
                }
                system_prompt.push_str(&knowledge_context);
            }
        }

        // 4. 스타일팩(말투 프로필) 지침 병합 - 서버 동기화로 받은 활성 스타일이 있을 때만 반영
        let style_service = StyleService::new(self.conn);
        if let Ok(style_prompt) = style_service.get_assembled_style_prompt() {
            system_prompt.push_str(&style_prompt);
        }

        // 5. 대화 이력 수집
        let history = ChatRepository::list_messages(self.conn, &req.room_id)
            .map_err(|e| ChatError::Database(e.to_string()))?;

        // 6. LLM 추론 처리 (비즈니스 계층 침투 방지를 위해 클로저 위임 호출)
        let ai_text = llm_infer_fn(system_prompt, history)?;

        // 7. AI 응답 메시지 생성 및 SQLite 저장
        let ai_now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        let ai_msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            room_id: req.room_id,
            role: "assistant".to_string(),
            content: ai_text,
            created_at: ai_now,
        };
        ChatRepository::insert_message(self.conn, &ai_msg)
            .map_err(|e| ChatError::Database(e.to_string()))?;

        Ok(ai_msg)
    }
}
