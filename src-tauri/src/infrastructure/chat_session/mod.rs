use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::model::LlamaLoraAdapter;
use llama_cpp_2::token::LlamaToken;

use crate::infrastructure::cache::CacheController;
use crate::infrastructure::i18n::pick;
use crate::infrastructure::llm::scheduler::RequestRegistry;
use crate::infrastructure::llm::streaming::StreamTarget;
use crate::infrastructure::llm::{
    CacheGenerationResult, GenerationRuntime, LlmEngine, LlmError, EPHEMERAL_CONTEXT_SIZE,
};

#[derive(Debug, Clone)]
pub struct SessionGenerationStats {
    pub prompt_tokens: usize,
    pub cached_tokens: usize,
    pub generated_tokens: usize,
    pub reused_prefix_tokens: usize,
    pub truncated_prompt_tokens: usize,
    pub cache_reset: bool,
}

#[derive(Debug, Clone)]
pub struct ChatSessionStatus {
    pub persona_id: String,
    pub cached_tokens: usize,
    pub lora_adapter_mounted: bool,
    pub last_access: u64,
    pub last_generation: Option<SessionGenerationStats>,
}

struct PersonaSession<'a> {
    context: LlamaContext<'a>,
    lora_adapter: Option<LlamaLoraAdapter>,
    cached_tokens: Vec<LlamaToken>,
    last_access: u64,
    last_generation: Option<SessionGenerationStats>,
}

fn session_missing_error(language: &str, persona_id: &str) -> LlmError {
    LlmError::infer(
        language,
        &pick(
            language,
            format!("LLM 세션 생성 실패: {persona_id}"),
            format!("Failed to create LLM session: {persona_id}"),
            format!("LLM 会话创建失败：{persona_id}"),
        ),
    )
}

fn cancelled_error(language: &str) -> LlmError {
    LlmError::infer(
        language,
        &pick(
            language,
            "추론 요청이 취소되었습니다.".to_string(),
            "The inference request was cancelled.".to_string(),
            "推理请求已取消。".to_string(),
        ),
    )
}

fn session_recovery_error(language: &str, first_error: &LlmError, second_error: &LlmError) -> LlmError {
    LlmError::infer(
        language,
        &pick(
            language,
            format!(
                "세션 캐시 복구 실패: 최초 오류: {}; 재시도 오류: {}",
                first_error, second_error
            ),
            format!(
                "Failed to recover session cache: first error: {}; retry error: {}",
                first_error, second_error
            ),
            format!(
                "会话缓存恢复失败：首次错误：{}；重试错误：{}",
                first_error, second_error
            ),
        ),
    )
}

/// LLM 워커 스레드가 소유하는 정령별 대화 세션(KV 캐시가 실린 LlamaContext)의
/// 생성·조회·LRU 축출·추론 오케스트레이션을 전담하는 채팅세션관리자.
/// 로컬모델관리자(LlmEngine)에서 순수 모델 로딩/추론 계산 책임을 떼어내
/// "세션 생명주기"라는 단일 도메인으로 분리한다.
pub struct ChatSessionController<'a> {
    engine: &'a LlmEngine,
    cache: Arc<CacheController>,
    sessions: HashMap<String, PersonaSession<'a>>,
    access_counter: u64,
}

impl<'a> ChatSessionController<'a> {
    pub fn new(engine: &'a LlmEngine, cache: Arc<CacheController>) -> Self {
        Self {
            engine,
            cache,
            sessions: HashMap::new(),
            access_counter: 0,
        }
    }

    fn generation_stats(result: &CacheGenerationResult) -> SessionGenerationStats {
        SessionGenerationStats {
            prompt_tokens: result.prompt_tokens,
            cached_tokens: result.cached_tokens.len(),
            generated_tokens: result.generated_tokens,
            reused_prefix_tokens: result.reused_prefix_tokens,
            truncated_prompt_tokens: result.truncated_prompt_tokens,
            cache_reset: result.cache_reset,
        }
    }

    fn create_session(&self, persona_id: &str, last_access: u64) -> Result<PersonaSession<'a>, LlmError> {
        let mut context = self.engine.create_context()?;
        let lora_adapter = self.engine.mount_lora_adapter(&mut context, persona_id)?;
        Ok(PersonaSession {
            context,
            lora_adapter,
            cached_tokens: Vec::new(),
            last_access,
            last_generation: None,
        })
    }

    fn ensure_session(&mut self, persona_id: &str) -> Result<u64, LlmError> {
        if !self.sessions.contains_key(persona_id)
            && self.sessions.len() >= self.engine.profile().max_active_sessions
        {
            if let Some(oldest_id) = self
                .sessions
                .iter()
                .min_by_key(|(_, session)| session.last_access)
                .map(|(id, _)| id.clone())
            {
                if let Some(evicted) = self.sessions.remove(&oldest_id) {
                    let cache_path = self.cache.session_cache_path(&oldest_id);
                    let _ = evicted.context.state_save_file(&cache_path, &evicted.cached_tokens);
                }
            }
        }

        self.access_counter += 1;
        let current_access = self.access_counter;

        if !self.sessions.contains_key(persona_id) {
            self.sessions.insert(
                persona_id.to_string(),
                self.create_session(persona_id, current_access)?,
            );
        }

        Ok(current_access)
    }

    pub fn warm_persona(
        &mut self,
        persona_id: &str,
        prompt: &str,
        mut progress_callback: impl FnMut(usize, usize),
    ) -> Result<SessionGenerationStats, LlmError> {
        let current_access = self.ensure_session(persona_id)?;
        let engine = self.engine;
        let cache_path = self.cache.session_cache_path(persona_id);

        let session = self
            .sessions
            .get_mut(persona_id)
            .ok_or_else(|| session_missing_error(engine.language(), persona_id))?;
        session.last_access = current_access;

        if session.cached_tokens.is_empty() {
            if let Ok(loaded_tokens) = session.context.state_load_file(&cache_path, 4096) {
                session.cached_tokens = loaded_tokens;
            }
        }

        let mut runtime = GenerationRuntime {
            cancel_flag: None,
            token_callback: None,
            progress_callback: Some(&mut progress_callback),
            language: engine.language().to_string(),
        };

        let result = engine.prefill_cache_runtime(
            &mut session.context,
            &session.cached_tokens,
            prompt,
            &mut runtime,
        )?;
        let stats = Self::generation_stats(&result);
        session.last_generation = Some(stats.clone());
        session.cached_tokens = result.cached_tokens;

        let _ = session.context.state_save_file(&cache_path, &session.cached_tokens);

        Ok(stats)
    }

    fn infer_in_session(
        &mut self,
        persona_id: &str,
        request_registry: &RequestRegistry,
        request_id: &str,
        cancel_flag: &Arc<AtomicBool>,
        stream: Option<StreamTarget>,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<String, LlmError> {
        let current_access = self.ensure_session(persona_id)?;
        let engine = self.engine;
        let cache_path = self.cache.session_cache_path(persona_id);

        {
            let session = self
                .sessions
                .get_mut(persona_id)
                .ok_or_else(|| session_missing_error(engine.language(), persona_id))?;
            session.last_access = current_access;
        }

        let mut callback = |token: String| -> Result<(), LlmError> {
            if let Some(ref target) = stream {
                target.emit_token(request_id, token)?;
            }
            Ok(())
        };
        let mut runtime = GenerationRuntime {
            cancel_flag: Some(cancel_flag.as_ref()),
            token_callback: Some(&mut callback),
            progress_callback: None,
            language: engine.language().to_string(),
        };

        let session = self
            .sessions
            .get_mut(persona_id)
            .ok_or_else(|| session_missing_error(engine.language(), persona_id))?;

        match engine.generate_with_cache_runtime(
            &mut session.context,
            &session.cached_tokens,
            prompt,
            max_tokens,
            &mut runtime,
        ) {
            Ok(result) => {
                let text = result.text.clone();
                session.last_generation = Some(Self::generation_stats(&result));
                request_registry.update_generation(request_id, &result);
                session.cached_tokens = result.cached_tokens;

                let _ = session.context.state_save_file(&cache_path, &session.cached_tokens);

                if let Some(target) = stream {
                    target.emit_done(request_id, false, None);
                }
                Ok(text)
            }
            Err(first_error) => {
                if let Some(target) = stream {
                    target.emit_done(
                        request_id,
                        cancel_flag.load(Ordering::SeqCst),
                        Some(first_error.to_string()),
                    );
                    return Err(first_error);
                }

                let mut rebuilt_session = self.create_session(persona_id, current_access)?;
                let result = engine
                    .generate_with_cache_runtime(
                        &mut rebuilt_session.context,
                        &rebuilt_session.cached_tokens,
                        prompt,
                        max_tokens,
                        &mut runtime,
                    )
                    .map_err(|second_error| {
                        session_recovery_error(engine.language(), &first_error, &second_error)
                    })?;

                let text = result.text.clone();
                rebuilt_session.last_generation = Some(Self::generation_stats(&result));
                request_registry.update_generation(request_id, &result);
                rebuilt_session.cached_tokens = result.cached_tokens;
                let _ = rebuilt_session
                    .context
                    .state_save_file(&cache_path, &rebuilt_session.cached_tokens);
                self.sessions.insert(persona_id.to_string(), rebuilt_session);
                Ok(text)
            }
        }
    }

    pub fn infer(
        &mut self,
        request_registry: &RequestRegistry,
        request_id: &str,
        cancel_flag: &Arc<AtomicBool>,
        persona_id: Option<String>,
        prompt: &str,
        max_tokens: Option<u32>,
        stream: Option<StreamTarget>,
    ) -> Result<String, LlmError> {
        if cancel_flag.load(Ordering::SeqCst) {
            return Err(cancelled_error(self.engine.language()));
        }
        let max_tokens = max_tokens.unwrap_or(self.engine.profile().max_tokens);

        let Some(persona_id) = persona_id else {
            let ephemeral_size = EPHEMERAL_CONTEXT_SIZE.min(self.engine.profile().context_size);
            let mut ctx = self.engine.create_context_with_size(ephemeral_size)?;
            return self.engine.generate_on_context(&mut ctx, prompt, max_tokens);
        };

        self.infer_in_session(
            &persona_id,
            request_registry,
            request_id,
            cancel_flag,
            stream,
            prompt,
            max_tokens,
        )
    }

    pub fn active_session_ids(&self) -> Vec<String> {
        let mut entries: Vec<(String, u64)> = self
            .sessions
            .iter()
            .map(|(id, session)| (id.clone(), session.last_access))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.into_iter().map(|(id, _)| id).collect()
    }

    pub fn statuses(&self) -> Vec<ChatSessionStatus> {
        let mut statuses: Vec<ChatSessionStatus> = self
            .sessions
            .iter()
            .map(|(persona_id, session)| ChatSessionStatus {
                persona_id: persona_id.clone(),
                cached_tokens: session.cached_tokens.len(),
                lora_adapter_mounted: session.lora_adapter.is_some(),
                last_access: session.last_access,
                last_generation: session.last_generation.clone(),
            })
            .collect();
        statuses.sort_by(|a, b| b.last_access.cmp(&a.last_access));
        statuses
    }
}
