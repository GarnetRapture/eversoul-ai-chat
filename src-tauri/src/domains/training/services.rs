use tauri::{AppHandle, Emitter};

use crate::domains::auth::commands::DbState;
use crate::domains::persona::repositories::PersonaRepository;
use crate::infrastructure::llm::get_model_relative_path;
use crate::infrastructure::training::{train_persona_lora, ConversationExample};
use super::types::{TrainingError, TrainingProgress, TrainingSummary};

const MIN_TRAINING_EXAMPLES: usize = 5;
const MAX_SOURCE_MESSAGES: i64 = 1000;

/// lib.rs의 setup 로직과 동일한 exe 기준 앱 루트 산정 방식.
/// 디버그 빌드는 작업 디렉토리, 릴리즈 빌드는 실행 파일이 위치한 디렉토리를 기준으로 삼는다.
fn app_root_dir() -> std::path::PathBuf {
    #[cfg(debug_assertions)]
    {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    }
    #[cfg(not(debug_assertions))]
    {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }
}

pub async fn run_training(
    persona_id: String,
    app_handle: AppHandle,
    db_state: &DbState,
    adapters_dir: &std::path::Path,
    active_model: &str,
    language: &str,
) -> Result<TrainingSummary, TrainingError> {
    // 1. 페르소나 시스템 프롬프트 조회
    let system_prompt = {
        let conn = db_state
            .0
            .get()
            .map_err(|_| TrainingError::db_lock(language))?;
        let persona = PersonaRepository::get_persona(&conn, &persona_id)
            .map_err(|_| TrainingError::query_failed(language))?
            .ok_or_else(|| TrainingError::persona_not_found(language, &persona_id))?;
        persona.system_prompt
    };

    // 2. 코퍼스 준비를 위해 DB에서 채팅 기록 추출 (chat_message 테이블,
    //    chat_message.persona_id가 비어 있으면 chat_room.persona_id로 폴백)
    let rows: Vec<(String, String)> = {
        let conn = db_state
            .0
            .get()
            .map_err(|_| TrainingError::db_lock(language))?;
        let mut stmt = conn
            .prepare(
                "SELECT cm.role, cm.content
                 FROM chat_message cm
                 JOIN chat_room cr ON cr.id = cm.room_id
                 WHERE cm.persona_id = ?1 OR (cm.persona_id IS NULL AND cr.persona_id = ?1)
                 ORDER BY cm.created_at ASC
                 LIMIT ?2",
            )
            .map_err(|_| TrainingError::query_failed(language))?;

        let messages = stmt
            .query_map(rusqlite::params![&persona_id, MAX_SOURCE_MESSAGES], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|_| TrainingError::query_failed(language))?;

        messages.filter_map(Result::ok).collect()
    };

    // 3. 시간순 대화를 (user/assistant 히스토리 -> assistant 응답) 학습 예시로 변환
    let mut examples: Vec<ConversationExample> = Vec::new();
    let mut history: Vec<(String, String)> = Vec::new();
    for (role, content) in rows {
        if role == "system" {
            continue;
        }
        if role == "assistant" && !history.is_empty() {
            examples.push(ConversationExample {
                system_prompt: system_prompt.clone(),
                prompt_turns: history.clone(),
                target_reply: content.clone(),
            });
        }
        history.push((role, content));
    }

    if examples.len() < MIN_TRAINING_EXAMPLES {
        return Err(TrainingError::insufficient_data(
            language,
            MIN_TRAINING_EXAMPLES,
            examples.len(),
        ));
    }
    let examples_used = examples.len();

    // 4. 실제 Rust(candle) LoRA 학습 실행. 원본 가중치는 채팅 추론과 동일한, 이미
    //    로컬에 고정 배치된 활성 GGUF 모델(ai/model/*.gguf)을 그대로 역양자화해서 쓴다.
    //    새 원본 모델을 인터넷에서 별도로 받지 않는다. 어댑터 GGUF는 mount_lora_adapter가
    //    찾는 adapters_dir/{persona_id}.gguf 경로와 정확히 일치해야 하므로, candle 원본
    //    가중치는 같은 스템의 .bin으로 저장해 with_extension("gguf")가 그 경로를 가리키게 한다.
    let app_root = app_root_dir();
    let base_model_gguf_path = app_root.join(get_model_relative_path(active_model));
    let weights_path = adapters_dir.join(format!("{persona_id}.bin"));
    let progress_persona_id = persona_id.clone();
    let progress_app_handle = app_handle.clone();

    let report = tauri::async_runtime::spawn_blocking(move || {
        train_persona_lora(
            examples,
            &base_model_gguf_path,
            &weights_path,
            |step, total_steps, loss| {
                let progress = TrainingProgress {
                    persona_id: progress_persona_id.clone(),
                    step,
                    total_steps,
                    loss,
                };
                let _ = progress_app_handle.emit("training-progress", &progress);
            },
        )
    })
    .await
    .map_err(|e| TrainingError::thread_panic(language, &e.to_string()))?
    .map_err(|e| {
        let detail = e.to_string();
        if let Some(rest) = detail.strip_prefix("architecture_mismatch:") {
            let mut parts = rest.splitn(2, ':');
            let expected = parts.next().unwrap_or_default();
            let found = parts.next().unwrap_or_default();
            TrainingError::architecture_mismatch(language, expected, found)
        } else {
            TrainingError::training_failed(language, &detail)
        }
    })?;

    Ok(TrainingSummary {
        persona_id,
        examples_used,
        steps: report.steps,
        final_loss: report.final_loss,
    })
}
