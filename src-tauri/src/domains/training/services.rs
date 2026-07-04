use rusqlite::Connection;

use super::types::TrainingError;
use crate::domains::chat::repositories::ChatRepository;
use crate::domains::persona::repositories::PersonaRepository;
use crate::infrastructure::training::ConversationExample;

pub const MIN_EXAMPLES: usize = 5;

pub struct TrainingService;

impl TrainingService {

    pub fn collect_training_examples(
        conn: &Connection,
        persona_id: &str,
    ) -> Result<Vec<ConversationExample>, TrainingError> {
        let persona = PersonaRepository::get_persona(conn, persona_id)
            .map_err(|e| TrainingError::Database(e.to_string()))?
            .ok_or_else(|| TrainingError::Database(format!("정령을 찾을 수 없습니다: {persona_id}")))?;

        let rooms = ChatRepository::list_rooms_by_persona(conn, persona_id)
            .map_err(|e| TrainingError::Database(e.to_string()))?;

        let mut examples = Vec::new();
        for room in rooms {
            let messages = ChatRepository::list_messages(conn, &room.id)
                .map_err(|e| TrainingError::Database(e.to_string()))?;

            let mut history: Vec<(String, String)> = Vec::new();
            for msg in &messages {
                if msg.role == "assistant" {
                    examples.push(ConversationExample {
                        system_prompt: persona.system_prompt.clone(),
                        prompt_turns: history.clone(),
                        target_reply: msg.content.clone(),
                    });
                }
                history.push((msg.role.clone(), msg.content.clone()));
            }
        }

        if examples.len() < MIN_EXAMPLES {
            return Err(TrainingError::InsufficientData(format!(
                "누적된 대화 예시가 {}건뿐입니다(최소 {}건 필요). 대화를 더 나눈 뒤 다시 시도하십시오.",
                examples.len(),
                MIN_EXAMPLES
            )));
        }

        Ok(examples)
    }
}
