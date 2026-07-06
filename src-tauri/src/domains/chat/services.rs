use rusqlite::Connection;
use std::time::SystemTime;
use uuid::Uuid;

use super::repositories::ChatRepository;
use super::types::{ChatError, ChatMessage, ChatRoom, SendMessageRequest};
use crate::domains::knowledge::services::KnowledgeService;
use crate::domains::persona::services::PersonaService;
use crate::domains::style::services::StyleService;
use crate::infrastructure::llm::worker::LlmWorkerHandle;
use crate::infrastructure::settings::SettingsManager;

const EPISODIC_INJECT_LIMIT: usize = 5;

const EPISODIC_SEARCH_CANDIDATE_LIMIT: usize = 80;

const PROMPT_HISTORY_LIMIT: usize = 10;

const CONSOLIDATION_INTERVAL: usize = 10;

const CONSOLIDATION_SOURCE_LIMIT: usize = 30;

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

    pub fn get_room_messages(&self, room_id: &str) -> Result<Vec<ChatMessage>, ChatError> {
        ChatRepository::list_messages(self.conn, room_id)
            .map_err(|e| ChatError::Database(e.to_string()))
    }

    pub fn prepare_message_context(
        &self,
        req: &SendMessageRequest,
        settings: &SettingsManager,
        query_vector: &[f32],
    ) -> Result<(String, Vec<ChatMessage>), ChatError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        let user_msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            room_id: req.room_id.clone(),
            role: "user".to_string(),
            content: req.content.clone(),
            created_at: now,
        };
        ChatRepository::insert_message(self.conn, &user_msg)
            .map_err(|e| ChatError::Database(e.to_string()))?;

        let persona_service = PersonaService::new(self.conn);
        let language = settings.get_language();
        let mut system_prompt = persona_service
            .get_assembled_system_prompt(&req.persona_id, &language)
            .map_err(|e| ChatError::Database(e))?;

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

        let style_service = StyleService::new(self.conn);
        let active_style_id = settings.get_active_style_id();
        if let Ok(style_prompt) =
            style_service.get_assembled_style_prompt(active_style_id.as_deref())
        {
            system_prompt.push_str(&style_prompt);
        }

        let semantic_memory = ChatRepository::get_semantic_memory(self.conn, &req.persona_id)
            .map_err(|e| ChatError::Database(e.to_string()))?;
        let relevant_episodic = ChatRepository::search_episodic_memories(
            self.conn,
            &req.persona_id,
            query_vector,
            EPISODIC_INJECT_LIMIT,
            EPISODIC_SEARCH_CANDIDATE_LIMIT,
        )
        .map_err(|e| ChatError::Database(e.to_string()))?;

        if semantic_memory.is_some() || !relevant_episodic.is_empty() {
            let mut memory_block =
                String::from("\n[구원자와의 관계에 대해 이 정령이 누적한 기억]\n");
            if let Some(ref summary) = semantic_memory {
                memory_block.push_str(&format!("- (통합 요약) {}\n", summary));
            }
            for note in &relevant_episodic {
                memory_block.push_str(&format!("- (관련 기억) {}\n", note));
            }
            system_prompt.push_str(&memory_block);
        }

        let history =
            ChatRepository::list_recent_messages(self.conn, &req.room_id, PROMPT_HISTORY_LIMIT)
                .map_err(|e| ChatError::Database(e.to_string()))?;

        Ok((system_prompt, history))
    }

    fn render_chat_message(msg: &ChatMessage) -> String {
        format!("<|im_start|>{}\n{}<|im_end|>\n", msg.role, msg.content)
    }

    pub fn build_llm_chat_prompt(system_prompt: &str, history: &[ChatMessage]) -> String {
        let mut full_prompt = String::new();
        full_prompt.push_str(&format!(
            "<|im_start|>system\n{}<|im_end|>\n",
            system_prompt
        ));

        for msg in history {
            full_prompt.push_str(&Self::render_chat_message(msg));
        }

        full_prompt.push_str("<|im_start|>assistant\n");
        full_prompt
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

    pub fn save_ai_response(&self, room_id: &str, text: String) -> Result<ChatMessage, ChatError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        let ai_msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            room_id: room_id.to_string(),
            role: "assistant".to_string(),
            content: text,
            created_at: now,
        };
        ChatRepository::insert_message(self.conn, &ai_msg)
            .map_err(|e| ChatError::Database(e.to_string()))?;

        Ok(ai_msg)
    }

    pub fn accumulate_memory(
        &self,
        persona_id: &str,
        user_text: &str,
        ai_text: &str,
        engine: &LlmWorkerHandle,
    ) -> Result<(), ChatError> {
        let extraction_prompt = format!(
            "<|im_start|>system\n다음은 정령 캐릭터와 구원자(사용자) 사이의 대화 한 턴이다. \
             이 캐릭터가 앞으로의 대화에서 계속 기억해야 할 새로운 사실, 사건, 구원자의 취향/약속 \
             등이 있다면 한국어 한 문장으로 간결하게 요약하라. 기억할 만한 새로운 정보가 없으면 \
             정확히 '없음'이라고만 답하라.<|im_end|>\n\
             <|im_start|>user\n구원자: {}\n캐릭터: {}<|im_end|>\n\
             <|im_start|>assistant\n",
            user_text, ai_text
        );

        let summary = engine
            .infer(&extraction_prompt, Some(80), None)
            .map_err(|e| ChatError::LlmInferenceFailed(e.to_string()))?;
        let trimmed = summary.trim();

        if !trimmed.is_empty() && !trimmed.contains("없음") {
            let memory_vector = engine
                .embed_text(trimmed)
                .map_err(|e| ChatError::LlmInferenceFailed(e.to_string()))?;
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();
            ChatRepository::insert_episodic_memory(
                self.conn,
                &Uuid::new_v4().to_string(),
                persona_id,
                trimmed,
                &memory_vector,
                &now,
            )
            .map_err(|e| ChatError::Database(e.to_string()))?;
        }

        let episodic_count = ChatRepository::count_episodic_memories(self.conn, persona_id)
            .map_err(|e| ChatError::Database(e.to_string()))?;
        if episodic_count > 0 && episodic_count % CONSOLIDATION_INTERVAL == 0 {
            self.consolidate_semantic_memory(persona_id, engine)?;
        }

        Ok(())
    }

    fn consolidate_semantic_memory(
        &self,
        persona_id: &str,
        engine: &LlmWorkerHandle,
    ) -> Result<(), ChatError> {
        let episodic = ChatRepository::list_episodic_memories(
            self.conn,
            persona_id,
            CONSOLIDATION_SOURCE_LIMIT,
        )
        .map_err(|e| ChatError::Database(e.to_string()))?;
        if episodic.is_empty() {
            return Ok(());
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

        let new_summary = engine
            .infer(&consolidation_prompt, Some(200), None)
            .map_err(|e| ChatError::LlmInferenceFailed(e.to_string()))?;
        let trimmed = new_summary.trim();

        if !trimmed.is_empty() {
            let memory_vector = engine
                .embed_text(trimmed)
                .map_err(|e| ChatError::LlmInferenceFailed(e.to_string()))?;
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();
            ChatRepository::upsert_semantic_memory(
                self.conn,
                persona_id,
                trimmed,
                &memory_vector,
                &now,
            )
            .map_err(|e| ChatError::Database(e.to_string()))?;
        }

        Ok(())
    }
}
