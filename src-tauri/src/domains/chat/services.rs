use rusqlite::Connection;
use std::time::SystemTime;
use uuid::Uuid;

use super::repositories::ChatRepository;
use super::types::{ChatError, ChatMessage, ChatRoom, SendMessageRequest};
use crate::domains::knowledge::services::KnowledgeService;
use crate::domains::persona::services::PersonaService;
use crate::domains::style::services::StyleService;
use crate::infrastructure::settings::SettingsManager;

const EPISODIC_INJECT_LIMIT: usize = 5;

const PROMPT_HISTORY_LIMIT: usize = 6;

pub const CHAT_RESPONSE_MAX_TOKENS: u32 = 96;

const CONSOLIDATION_INTERVAL: usize = 10;

const CONSOLIDATION_SOURCE_LIMIT: usize = 30;

pub const CONSOLIDATION_MAX_TOKENS: u32 = 200;

pub const EVERTALK_SESSION_TITLE: &str = "EverTalk Session";

pub struct ChatService<'a> {
    conn: &'a Connection,
}

impl<'a> ChatService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create_chat_room(&self, title: &str) -> Result<ChatRoom, ChatError> {
        self.create_chat_session_room(title, None)
    }

    pub fn create_chat_session_room(
        &self,
        title: &str,
        persona_id: Option<String>,
    ) -> Result<ChatRoom, ChatError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        let room = ChatRoom {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            persona_id,
            session_started_at: now.clone(),
            created_at: now.clone(),
            updated_at: now,
        };

        ChatRepository::create_room(self.conn, &room)
            .map(|_| room)
            .map_err(|e| ChatError::Database(e.to_string()))
    }

    pub fn get_chat_rooms(&self) -> Result<Vec<ChatRoom>, ChatError> {
        ChatRepository::list_rooms(self.conn).map_err(|e| ChatError::Database(e.to_string()))
    }

    pub fn get_latest_session_room(&self, persona_id: &str) -> Result<Option<ChatRoom>, ChatError> {
        ChatRepository::find_latest_room_by_persona(self.conn, persona_id)
            .map_err(|e| ChatError::Database(e.to_string()))
    }

    pub fn get_or_create_evertalk_session_room(&self) -> Result<ChatRoom, ChatError> {
        if let Some(room) =
            ChatRepository::find_latest_global_session_room(self.conn, EVERTALK_SESSION_TITLE)
                .map_err(|e| ChatError::Database(e.to_string()))?
        {
            return Ok(room);
        }

        self.create_chat_session_room(EVERTALK_SESSION_TITLE, None)
    }

    pub fn get_room_messages(&self, room_id: &str) -> Result<Vec<ChatMessage>, ChatError> {
        ChatRepository::list_messages(self.conn, room_id)
            .map_err(|e| ChatError::Database(e.to_string()))
    }

    pub fn get_room_messages_for_persona(
        &self,
        room_id: &str,
        persona_id: &str,
    ) -> Result<Vec<ChatMessage>, ChatError> {
        ChatRepository::list_messages_for_persona(self.conn, room_id, persona_id)
            .map_err(|e| ChatError::Database(e.to_string()))
    }

    pub fn prepare_message_context(
        &self,
        req: &SendMessageRequest,
        settings: &SettingsManager,
    ) -> Result<(String, Vec<ChatMessage>), ChatError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        let user_msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            room_id: req.room_id.clone(),
            persona_id: Some(req.persona_id.clone()),
            role: "user".to_string(),
            content: req.content.clone(),
            created_at: now,
        };
        ChatRepository::insert_message(self.conn, &user_msg)
            .map_err(|e| ChatError::Database(e.to_string()))?;

        let mut system_prompt = self.build_persona_base_system_prompt(&req.persona_id, settings)?;

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

        let history = ChatRepository::list_recent_messages_for_persona(
            self.conn,
            &req.room_id,
            &req.persona_id,
            PROMPT_HISTORY_LIMIT,
        )
        .map_err(|e| ChatError::Database(e.to_string()))?;

        Ok((system_prompt, history))
    }

    pub fn build_persona_base_system_prompt(
        &self,
        persona_id: &str,
        settings: &SettingsManager,
    ) -> Result<String, ChatError> {
        let persona_service = PersonaService::new(self.conn);
        let language = settings.get_language();
        let mut system_prompt = persona_service
            .get_assembled_system_prompt(persona_id, &language)
            .map_err(ChatError::Database)?;

        let style_service = StyleService::new(self.conn);
        let active_style_id = settings.get_active_style_id();
        if let Ok(style_prompt) =
            style_service.get_assembled_style_prompt(active_style_id.as_deref())
        {
            system_prompt.push_str(&style_prompt);
        }

        let semantic_memory = ChatRepository::get_semantic_memory(self.conn, persona_id)
            .map_err(|e| ChatError::Database(e.to_string()))?;
        let recent_episodic =
            ChatRepository::list_episodic_memories(self.conn, persona_id, EPISODIC_INJECT_LIMIT)
                .map_err(|e| ChatError::Database(e.to_string()))?;

        if semantic_memory.is_some() || !recent_episodic.is_empty() {
            let mut memory_block =
                String::from("\n[구원자와의 관계에 대해 이 정령이 누적한 기억]\n");
            if let Some(ref summary) = semantic_memory {
                memory_block.push_str(&format!("- (통합 요약) {}\n", summary));
            }
            for note in &recent_episodic {
                memory_block.push_str(&format!("- (최근 기억) {}\n", note));
            }
            system_prompt.push_str(&memory_block);
        }

        Ok(system_prompt)
    }

    fn render_chat_message(msg: &ChatMessage) -> String {
        format!("<|im_start|>{}\n{}<|im_end|>\n", msg.role, msg.content)
    }

    pub fn build_llm_chat_prompt(system_prompt: &str, history: &[ChatMessage]) -> String {
        let mut full_prompt = String::new();
        full_prompt.push_str(&Self::build_llm_system_prefix(system_prompt));

        for msg in history {
            full_prompt.push_str(&Self::render_chat_message(msg));
        }

        full_prompt.push_str("<|im_start|>assistant\n");
        full_prompt
    }

    pub fn build_llm_system_prefix(system_prompt: &str) -> String {
        format!("<|im_start|>system\n{}<|im_end|>\n", system_prompt)
    }

    pub fn build_llm_chat_prompt_with_budget<F>(
        system_prompt: &str,
        history: &[ChatMessage],
        max_prompt_tokens: usize,
        mut count_tokens: F,
    ) -> Result<String, ChatError>
    where
        F: FnMut(&str) -> Result<usize, ChatError>,
    {
        let system_block = format!("<|im_start|>system\n{}<|im_end|>\n", system_prompt);
        let assistant_block = "<|im_start|>assistant\n";
        let base_prompt = format!("{}{}", system_block, assistant_block);

        if count_tokens(&base_prompt)? >= max_prompt_tokens {
            return Ok(base_prompt);
        }

        let mut selected_blocks: Vec<String> = Vec::new();
        for msg in history.iter().rev() {
            let block = Self::render_chat_message(msg);
            let mut candidate = String::new();
            candidate.push_str(&system_block);
            for selected in selected_blocks.iter().rev() {
                candidate.push_str(selected);
            }
            candidate.push_str(&block);
            candidate.push_str(assistant_block);

            if count_tokens(&candidate)? > max_prompt_tokens {
                continue;
            }

            selected_blocks.push(block);
        }

        let mut full_prompt = String::new();
        full_prompt.push_str(&system_block);
        for block in selected_blocks.iter().rev() {
            full_prompt.push_str(block);
        }
        full_prompt.push_str(assistant_block);
        Ok(full_prompt)
    }

    pub fn save_ai_response(
        &self,
        room_id: &str,
        persona_id: &str,
        text: String,
    ) -> Result<ChatMessage, ChatError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        let ai_msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            room_id: room_id.to_string(),
            persona_id: Some(persona_id.to_string()),
            role: "assistant".to_string(),
            content: text,
            created_at: now,
        };
        ChatRepository::insert_message(self.conn, &ai_msg)
            .map_err(|e| ChatError::Database(e.to_string()))?;

        Ok(ai_msg)
    }

    pub fn record_turn_memory(
        &self,
        persona_id: &str,
        user_text: &str,
        ai_text: &str,
    ) -> Result<Option<String>, ChatError> {
        let trimmed_user = user_text.trim();
        let trimmed_ai = ai_text.trim();

        if !trimmed_user.is_empty() && !trimmed_ai.is_empty() {
            let memory_text = format!("구원자: {}\n정령: {}", trimmed_user, trimmed_ai);
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();
            ChatRepository::insert_episodic_memory(
                self.conn,
                &Uuid::new_v4().to_string(),
                persona_id,
                &memory_text,
                &[],
                &now,
            )
            .map_err(|e| ChatError::Database(e.to_string()))?;
        }

        let episodic_count = ChatRepository::count_episodic_memories(self.conn, persona_id)
            .map_err(|e| ChatError::Database(e.to_string()))?;
        if episodic_count == 0 || episodic_count % CONSOLIDATION_INTERVAL != 0 {
            return Ok(None);
        }

        self.build_consolidation_prompt(persona_id)
    }

    fn build_consolidation_prompt(
        &self,
        persona_id: &str,
    ) -> Result<Option<String>, ChatError> {
        let episodic = ChatRepository::list_episodic_memories(
            self.conn,
            persona_id,
            CONSOLIDATION_SOURCE_LIMIT,
        )
        .map_err(|e| ChatError::Database(e.to_string()))?;
        if episodic.is_empty() {
            return Ok(None);
        }

        let previous_summary = ChatRepository::get_semantic_memory(self.conn, persona_id)
            .map_err(|e| ChatError::Database(e.to_string()))?
            .unwrap_or_else(|| "없음".to_string());

        let episodic_list = episodic
            .iter()
            .enumerate()
            .map(|(i, m)| format!("{}. {}", i + 1, m))
            .collect::<Vec<String>>()
            .join("\n");

        let consolidation_prompt = format!(
            "<|im_start|>system\n다음은 정령 캐릭터가 구원자(사용자)와의 대화에서 그동안 기록해 \
             온 개별 기억들과, 이전에 정리했던 통합 요약이다. 이 모든 정보를 종합해 이 캐릭터가 \
             구원자에 대해 알고 있는 핵심 사실/취향/관계 상태를 한국어 3~5문장 이내로 새롭게 통합 \
             요약하라. 중복은 제거하고 최신 정보를 우선하라.<|im_end|>\n\
             <|im_start|>user\n[이전 통합 요약]\n{}\n\n[개별 기억 목록]\n{}<|im_end|>\n\
             <|im_start|>assistant\n",
            previous_summary, episodic_list
        );

        Ok(Some(consolidation_prompt))
    }

    pub fn store_semantic_summary(
        &self,
        persona_id: &str,
        summary: &str,
        summary_vector: &[f32],
    ) -> Result<(), ChatError> {
        let trimmed = summary.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();
        ChatRepository::upsert_semantic_memory(
            self.conn,
            persona_id,
            trimmed,
            summary_vector,
            &now,
        )
        .map_err(|e| ChatError::Database(e.to_string()))
    }
}
