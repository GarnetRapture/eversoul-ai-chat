use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::model::LlamaLoraAdapter;
use llama_cpp_2::token::LlamaToken;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::infrastructure::hardware::InferenceProfile;
use crate::startup_debug_log;

use super::scheduler::{LlmRequestStatus, RequestRegistry};
use super::streaming::StreamTarget;
use super::{
    CacheGenerationResult, GenerationRuntime, LlmEngine, LlmError, EPHEMERAL_CONTEXT_SIZE,
};

const LLM_WORKER_STACK_SIZE_BYTES: usize = 64 * 1024 * 1024;

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
pub struct WorkerSessionStatus {
    pub persona_id: String,
    pub cached_tokens: usize,
    pub lora_adapter_mounted: bool,
    pub last_access: u64,
    pub last_generation: Option<SessionGenerationStats>,
}

#[derive(Serialize, Clone)]
pub struct WarmProgressPayload {
    pub persona_id: String,
    pub current: usize,
    pub total: usize,
}

enum WorkerCommand {
    Infer {
        request_id: String,
        persona_id: Option<String>,
        prompt: String,
        max_tokens: Option<u32>,
        stream: Option<StreamTarget>,
        respond_to: Sender<Result<String, LlmError>>,
    },
    WarmPersona {
        persona_id: String,
        prompt: String,
        app_handle: AppHandle,
        respond_to: Sender<Result<SessionGenerationStats, LlmError>>,
    },
    Embed {
        text: String,
        respond_to: Sender<Result<Vec<f32>, LlmError>>,
    },
    CountTokens {
        text: String,
        respond_to: Sender<Result<usize, LlmError>>,
    },
    ActiveSessions {
        respond_to: Sender<Vec<String>>,
    },
    SessionStatuses {
        respond_to: Sender<Vec<WorkerSessionStatus>>,
    },
    RequestStatuses {
        respond_to: Sender<Vec<LlmRequestStatus>>,
    },
}

struct PersonaSession<'a> {
    context: LlamaContext<'a>,
    lora_adapter: Option<LlamaLoraAdapter>,
    cached_tokens: Vec<LlamaToken>,
    last_access: u64,
    last_generation: Option<SessionGenerationStats>,
}

#[derive(Clone)]
pub struct LlmWorkerHandle {
    sender: Sender<WorkerCommand>,
    model_path: PathBuf,
    profile: InferenceProfile,
    request_registry: RequestRegistry,
}

impl LlmWorkerHandle {
    pub fn spawn(engine: LlmEngine) -> Result<Self, LlmError> {
        let model_path = engine.model_path().to_path_buf();
        let profile = engine.profile();
        let (sender, receiver) = mpsc::channel::<WorkerCommand>();
        let request_registry = RequestRegistry::new();
        let worker_request_registry = request_registry.clone();

        thread::Builder::new()
            .name("eversoul-llm-worker".to_string())
            .stack_size(LLM_WORKER_STACK_SIZE_BYTES)
            .spawn(move || Self::run(engine, receiver, worker_request_registry))
            .map_err(|e| LlmError::BackendInit(format!("LLM 워커 스레드 생성 실패: {e}")))?;

        Ok(Self {
            sender,
            model_path,
            profile,
            request_registry,
        })
    }

    pub fn load_and_spawn(
        app_root: PathBuf,
        adapters_dir: PathBuf,
        profile: InferenceProfile,
    ) -> Result<Self, LlmError> {
        startup_debug_log("llm_worker:load_and_spawn:start");
        let model_path = app_root.join(super::MODEL_RELATIVE_PATH);
        let (sender, receiver) = mpsc::channel::<WorkerCommand>();
        let (ready_sender, ready_receiver) = mpsc::channel::<Result<(), LlmError>>();
        let request_registry = RequestRegistry::new();
        let worker_request_registry = request_registry.clone();

        startup_debug_log("llm_worker:load_and_spawn:thread_spawn:before");
        thread::Builder::new()
            .name("eversoul-llm-worker".to_string())
            .stack_size(LLM_WORKER_STACK_SIZE_BYTES)
            .spawn(move || {
                startup_debug_log("llm_worker:thread:started");
                startup_debug_log("llm_worker:thread:engine_load:start");
                let engine = match LlmEngine::load(&app_root, adapters_dir, profile) {
                    Ok(engine) => engine,
                    Err(error) => {
                        startup_debug_log("llm_worker:thread:engine_load:error");
                        let _ = ready_sender.send(Err(error));
                        return;
                    }
                };
                startup_debug_log("llm_worker:thread:engine_load:done");

                if ready_sender.send(Ok(())).is_err() {
                    startup_debug_log("llm_worker:thread:ready_send_failed");
                    return;
                }

                startup_debug_log("llm_worker:thread:run_loop:start");
                Self::run(engine, receiver, worker_request_registry);
                startup_debug_log("llm_worker:thread:run_loop:done");
            })
            .map_err(|e| LlmError::BackendInit(format!("LLM 워커 스레드 생성 실패: {e}")))?;
        startup_debug_log("llm_worker:load_and_spawn:thread_spawn:after");

        ready_receiver
            .recv()
            .map_err(|e| LlmError::BackendInit(format!("LLM 워커 초기화 응답 수신 실패: {e}")))??;
        startup_debug_log("llm_worker:load_and_spawn:ready");

        Ok(Self {
            sender,
            model_path,
            profile,
            request_registry,
        })
    }

    fn run(
        engine: LlmEngine,
        receiver: Receiver<WorkerCommand>,
        request_registry: RequestRegistry,
    ) {
        let mut sessions: HashMap<String, PersonaSession> = HashMap::new();
        let mut access_counter: u64 = 0;

        while let Ok(command) = receiver.recv() {
            match command {
                WorkerCommand::Infer {
                    request_id,
                    persona_id,
                    prompt,
                    max_tokens,
                    stream,
                    respond_to,
                } => {
                    let cancel_flag = request_registry.register(&request_id, persona_id.clone());
                    let result = Self::handle_infer(
                        &engine,
                        &mut sessions,
                        &mut access_counter,
                        &request_registry,
                        &request_id,
                        &cancel_flag,
                        persona_id,
                        &prompt,
                        max_tokens,
                        stream,
                    );
                    request_registry.finish(
                        &request_id,
                        &result,
                        cancel_flag.load(Ordering::SeqCst),
                    );
                    let _ = respond_to.send(result);
                }
                WorkerCommand::WarmPersona {
                    persona_id,
                    prompt,
                    app_handle,
                    respond_to,
                } => {
                    let result = Self::handle_warm_persona(
                        &engine,
                        &mut sessions,
                        &mut access_counter,
                        &persona_id,
                        &prompt,
                        app_handle,
                    );
                    let _ = respond_to.send(result);
                }
                WorkerCommand::Embed { text, respond_to } => {
                    let result = engine.embed_text(&text);
                    let _ = respond_to.send(result);
                }
                WorkerCommand::CountTokens { text, respond_to } => {
                    let result = engine.count_tokens(&text);
                    let _ = respond_to.send(result);
                }
                WorkerCommand::ActiveSessions { respond_to } => {
                    let mut entries: Vec<(String, u64)> = sessions
                        .iter()
                        .map(|(id, session)| (id.clone(), session.last_access))
                        .collect();
                    entries.sort_by(|a, b| b.1.cmp(&a.1));
                    let ids = entries.into_iter().map(|(id, _)| id).collect();
                    let _ = respond_to.send(ids);
                }
                WorkerCommand::SessionStatuses { respond_to } => {
                    let mut statuses: Vec<WorkerSessionStatus> = sessions
                        .iter()
                        .map(|(persona_id, session)| WorkerSessionStatus {
                            persona_id: persona_id.clone(),
                            cached_tokens: session.cached_tokens.len(),
                            lora_adapter_mounted: session.lora_adapter.is_some(),
                            last_access: session.last_access,
                            last_generation: session.last_generation.clone(),
                        })
                        .collect();
                    statuses.sort_by(|a, b| b.last_access.cmp(&a.last_access));
                    let _ = respond_to.send(statuses);
                }
                WorkerCommand::RequestStatuses { respond_to } => {
                    let _ = respond_to.send(request_registry.statuses());
                }
            }
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

    fn create_persona_session<'a>(
        engine: &'a LlmEngine,
        persona_id: &str,
        last_access: u64,
    ) -> Result<PersonaSession<'a>, LlmError> {
        let mut context = engine.create_context()?;
        let lora_adapter = engine.mount_lora_adapter(&mut context, persona_id)?;
        Ok(PersonaSession {
            context,
            lora_adapter,
            cached_tokens: Vec::new(),
            last_access,
            last_generation: None,
        })
    }

    fn ensure_persona_session<'a>(
        engine: &'a LlmEngine,
        sessions: &mut HashMap<String, PersonaSession<'a>>,
        access_counter: &mut u64,
        persona_id: &str,
    ) -> Result<u64, LlmError> {
        if !sessions.contains_key(persona_id)
            && sessions.len() >= engine.profile().max_active_sessions
        {
            if let Some(oldest_id) = sessions
                .iter()
                .min_by_key(|(_, session)| session.last_access)
                .map(|(id, _)| id.clone())
            {
                sessions.remove(&oldest_id);
            }
        }

        *access_counter += 1;
        let current_access = *access_counter;

        if !sessions.contains_key(persona_id) {
            sessions.insert(
                persona_id.to_string(),
                Self::create_persona_session(engine, persona_id, current_access)?,
            );
        }

        Ok(current_access)
    }

    fn handle_warm_persona<'a>(
        engine: &'a LlmEngine,
        sessions: &mut HashMap<String, PersonaSession<'a>>,
        access_counter: &mut u64,
        persona_id: &str,
        prompt: &str,
        app_handle: AppHandle,
    ) -> Result<SessionGenerationStats, LlmError> {
        let current_access =
            Self::ensure_persona_session(engine, sessions, access_counter, persona_id)?;
        let session = sessions
            .get_mut(persona_id)
            .ok_or_else(|| LlmError::Infer(format!("LLM 세션 생성 실패: {persona_id}")))?;
        session.last_access = current_access;

        let mut cache_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        cache_dir.push("ai");
        cache_dir.push("cache");
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_path = cache_dir.join(format!("{}.bin", persona_id));
        
        if session.cached_tokens.is_empty() {
            if let Ok(loaded_tokens) = session.context.state_load_file(&cache_path, 4096) {
                session.cached_tokens = loaded_tokens;
            }
        }

        let persona_id_clone = persona_id.to_string();
        let mut progress_callback = |processed: usize, total: usize| {
            let _ = app_handle.emit("warm-progress", WarmProgressPayload {
                persona_id: persona_id_clone.clone(),
                current: processed,
                total,
            });
        };

        let mut runtime = GenerationRuntime {
            cancel_flag: None,
            token_callback: None,
            progress_callback: Some(&mut progress_callback),
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

    fn infer_with_session_recovery<'a>(
        engine: &'a LlmEngine,
        persona_id: &str,
        session: &mut PersonaSession<'a>,
        request_registry: &RequestRegistry,
        request_id: &str,
        cancel_flag: &Arc<AtomicBool>,
        stream: Option<StreamTarget>,
        prompt: &str,
        max_tokens: u32,
        current_access: u64,
    ) -> Result<String, LlmError> {
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
        };
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

                let mut rebuilt_session =
                    Self::create_persona_session(engine, persona_id, current_access)?;
                let result = engine
                    .generate_with_cache_runtime(
                        &mut rebuilt_session.context,
                        &rebuilt_session.cached_tokens,
                        prompt,
                        max_tokens,
                        &mut runtime,
                    )
                    .map_err(|second_error| {
                        LlmError::Infer(format!(
                            "세션 캐시 복구 실패: 최초 오류: {}; 재시도 오류: {}",
                            first_error, second_error
                        ))
                    })?;

                let text = result.text.clone();
                rebuilt_session.last_generation = Some(Self::generation_stats(&result));
                request_registry.update_generation(request_id, &result);
                rebuilt_session.cached_tokens = result.cached_tokens;
                *session = rebuilt_session;
                Ok(text)
            }
        }
    }

    fn handle_infer<'a>(
        engine: &'a LlmEngine,
        sessions: &mut HashMap<String, PersonaSession<'a>>,
        access_counter: &mut u64,
        request_registry: &RequestRegistry,
        request_id: &str,
        cancel_flag: &Arc<AtomicBool>,
        persona_id: Option<String>,
        prompt: &str,
        max_tokens: Option<u32>,
        stream: Option<StreamTarget>,
    ) -> Result<String, LlmError> {
        if cancel_flag.load(Ordering::SeqCst) {
            return Err(LlmError::Infer("추론 요청이 취소되었습니다.".to_string()));
        }
        let max_tokens = max_tokens.unwrap_or(engine.profile().max_tokens);

        let Some(persona_id) = persona_id else {
            let ephemeral_size = EPHEMERAL_CONTEXT_SIZE.min(engine.profile().context_size);
            let mut ctx = engine.create_context_with_size(ephemeral_size)?;
            return engine.generate_on_context(&mut ctx, prompt, max_tokens);
        };

        let current_access =
            Self::ensure_persona_session(engine, sessions, access_counter, &persona_id)?;

        let session = sessions
            .get_mut(&persona_id)
            .ok_or_else(|| LlmError::Infer(format!("LLM 세션 생성 실패: {persona_id}")))?;
        session.last_access = current_access;

        Self::infer_with_session_recovery(
            engine,
            &persona_id,
            session,
            request_registry,
            request_id,
            cancel_flag,
            stream,
            prompt,
            max_tokens,
            current_access,
        )
    }

    pub fn infer(
        &self,
        prompt: &str,
        max_tokens: Option<u32>,
        persona_id: Option<&str>,
    ) -> Result<String, LlmError> {
        let request_id = Uuid::new_v4().to_string();
        self.infer_with_request(&request_id, prompt, max_tokens, persona_id, None)
    }

    pub fn infer_with_request(
        &self,
        request_id: &str,
        prompt: &str,
        max_tokens: Option<u32>,
        persona_id: Option<&str>,
        stream: Option<(AppHandle, String, String)>,
    ) -> Result<String, LlmError> {
        let (respond_to, response) = mpsc::channel();
        self.sender
            .send(WorkerCommand::Infer {
                request_id: request_id.to_string(),
                persona_id: persona_id.map(|id| id.to_string()),
                prompt: prompt.to_string(),
                max_tokens,
                stream: stream.map(|(app_handle, token_event, done_event)| {
                    StreamTarget::new(app_handle, token_event, done_event)
                }),
                respond_to,
            })
            .map_err(|e| LlmError::Infer(format!("LLM 워커 요청 전송 실패: {e}")))?;

        response
            .recv()
            .map_err(|e| LlmError::Infer(format!("LLM 워커 응답 수신 실패: {e}")))?
    }

    pub fn warm_persona(
        &self,
        persona_id: &str,
        prompt: &str,
        app_handle: AppHandle,
    ) -> Result<SessionGenerationStats, LlmError> {
        let (respond_to, response) = mpsc::channel();
        self.sender
            .send(WorkerCommand::WarmPersona {
                persona_id: persona_id.to_string(),
                prompt: prompt.to_string(),
                app_handle,
                respond_to,
            })
            .map_err(|e| LlmError::Infer(format!("LLM 워커 요청 전송 실패: {e}")))?;

        response
            .recv()
            .map_err(|e| LlmError::Infer(format!("LLM 워커 응답 수신 실패: {e}")))?
    }

    pub fn cancel_request(&self, request_id: &str) -> bool {
        self.request_registry.cancel(request_id)
    }

    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>, LlmError> {
        let (respond_to, response) = mpsc::channel();
        self.sender
            .send(WorkerCommand::Embed {
                text: text.to_string(),
                respond_to,
            })
            .map_err(|e| LlmError::Infer(format!("LLM 워커 요청 전송 실패: {e}")))?;

        response
            .recv()
            .map_err(|e| LlmError::Infer(format!("LLM 워커 응답 수신 실패: {e}")))?
    }

    pub fn count_tokens(&self, text: &str) -> Result<usize, LlmError> {
        let (respond_to, response) = mpsc::channel();
        self.sender
            .send(WorkerCommand::CountTokens {
                text: text.to_string(),
                respond_to,
            })
            .map_err(|e| LlmError::Infer(format!("LLM 워커 요청 전송 실패: {e}")))?;

        response
            .recv()
            .map_err(|e| LlmError::Infer(format!("LLM 워커 응답 수신 실패: {e}")))?
    }

    pub fn active_sessions(&self) -> Vec<String> {
        let (respond_to, response) = mpsc::channel();
        if self
            .sender
            .send(WorkerCommand::ActiveSessions { respond_to })
            .is_err()
        {
            return Vec::new();
        }
        response.recv().unwrap_or_default()
    }

    pub fn session_statuses(&self) -> Vec<WorkerSessionStatus> {
        let (respond_to, response) = mpsc::channel();
        if self
            .sender
            .send(WorkerCommand::SessionStatuses { respond_to })
            .is_err()
        {
            return Vec::new();
        }
        response.recv().unwrap_or_default()
    }

    pub fn request_statuses(&self) -> Vec<LlmRequestStatus> {
        let (respond_to, response) = mpsc::channel();
        if self
            .sender
            .send(WorkerCommand::RequestStatuses { respond_to })
            .is_err()
        {
            return Vec::new();
        }
        response.recv().unwrap_or_default()
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn profile(&self) -> InferenceProfile {
        self.profile
    }
}
