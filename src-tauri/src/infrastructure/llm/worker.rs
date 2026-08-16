use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::infrastructure::cache::CacheController;
use crate::infrastructure::chat_session::{ChatSessionController, ChatSessionStatus, SessionGenerationStats};
use crate::infrastructure::hardware::InferenceProfile;
use crate::infrastructure::i18n::pick;
use crate::startup_debug_log;

use super::scheduler::{LlmRequestStatus, RequestRegistry};
use super::streaming::StreamTarget;
use super::{LlmEngine, LlmError};

const LLM_WORKER_STACK_SIZE_BYTES: usize = 64 * 1024 * 1024;

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
        respond_to: Sender<Vec<ChatSessionStatus>>,
    },
    RequestStatuses {
        respond_to: Sender<Vec<LlmRequestStatus>>,
    },
}

#[derive(Clone)]
pub struct LlmWorkerHandle {
    sender: Sender<WorkerCommand>,
    model_path: PathBuf,
    profile: InferenceProfile,
    request_registry: RequestRegistry,
    language: String,
}

impl LlmWorkerHandle {
    pub fn load_and_spawn(
        app_root: PathBuf,
        adapters_dir: PathBuf,
        profile: InferenceProfile,
        model_relative_path: &str,
        language: &str,
        cache: Arc<CacheController>,
    ) -> Result<Self, LlmError> {
        startup_debug_log("llm_worker:load_and_spawn:start");
        let model_path = app_root.join(model_relative_path);
        let (sender, receiver) = mpsc::channel::<WorkerCommand>();
        let (ready_sender, ready_receiver) = mpsc::channel::<Result<(), LlmError>>();
        let request_registry = RequestRegistry::new();
        let worker_request_registry = request_registry.clone();

        startup_debug_log("llm_worker:load_and_spawn:thread_spawn:before");
        let model_relative_path_owned = model_relative_path.to_string();
        let language_owned = language.to_string();
        let thread_language = language_owned.clone();
        thread::Builder::new()
            .name("eversoul-llm-worker".to_string())
            .stack_size(LLM_WORKER_STACK_SIZE_BYTES)
            .spawn(move || {
                startup_debug_log("llm_worker:thread:started");
                startup_debug_log("llm_worker:thread:engine_load:start");
                let engine = match LlmEngine::load(
                    &app_root,
                    adapters_dir,
                    profile,
                    &model_relative_path_owned,
                    &thread_language,
                ) {
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
                Self::run(engine, receiver, worker_request_registry, cache);
                startup_debug_log("llm_worker:thread:run_loop:done");
            })
            .map_err(|e| {
                LlmError::backend_init(
                    &language_owned,
                    &pick(
                        &language_owned,
                        format!("LLM 워커 스레드 생성 실패: {e}"),
                        format!("Failed to spawn the LLM worker thread: {e}"),
                        format!("LLM 工作线程创建失败：{e}"),
                    ),
                )
            })?;
        startup_debug_log("llm_worker:load_and_spawn:thread_spawn:after");

        ready_receiver.recv().map_err(|e| {
            LlmError::backend_init(
                language,
                &pick(
                    language,
                    format!("LLM 워커 초기화 응답 수신 실패: {e}"),
                    format!("Failed to receive LLM worker init response: {e}"),
                    format!("接收 LLM 工作线程初始化响应失败：{e}"),
                ),
            )
        })??;
        startup_debug_log("llm_worker:load_and_spawn:ready");

        Ok(Self {
            sender,
            model_path,
            profile,
            request_registry,
            language: language.to_string(),
        })
    }

    fn run(
        engine: LlmEngine,
        receiver: Receiver<WorkerCommand>,
        request_registry: RequestRegistry,
        cache: Arc<CacheController>,
    ) {
        let mut session_ctrl = ChatSessionController::new(&engine, cache);

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
                    let result = session_ctrl.infer(
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
                        cancel_flag.load(std::sync::atomic::Ordering::SeqCst),
                    );
                    let _ = respond_to.send(result);
                }
                WorkerCommand::WarmPersona {
                    persona_id,
                    prompt,
                    app_handle,
                    respond_to,
                } => {
                    let persona_id_clone = persona_id.clone();
                    let result = session_ctrl.warm_persona(&persona_id, &prompt, |processed, total| {
                        let _ = app_handle.emit(
                            "warm-progress",
                            WarmProgressPayload {
                                persona_id: persona_id_clone.clone(),
                                current: processed,
                                total,
                            },
                        );
                    });
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
                    let _ = respond_to.send(session_ctrl.active_session_ids());
                }
                WorkerCommand::SessionStatuses { respond_to } => {
                    let _ = respond_to.send(session_ctrl.statuses());
                }
                WorkerCommand::RequestStatuses { respond_to } => {
                    let _ = respond_to.send(request_registry.statuses());
                }
            }
        }
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
                    StreamTarget::new(app_handle, token_event, done_event, self.language.clone())
                }),
                respond_to,
            })
            .map_err(|e| self.send_failed_error(&e.to_string()))?;

        response.recv().map_err(|e| self.recv_failed_error(&e.to_string()))?
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
            .map_err(|e| self.send_failed_error(&e.to_string()))?;

        response.recv().map_err(|e| self.recv_failed_error(&e.to_string()))?
    }

    pub fn cancel_request(&self, request_id: &str) -> bool {
        self.request_registry.cancel(request_id)
    }

    fn send_failed_error(&self, detail: &str) -> LlmError {
        LlmError::infer(
            &self.language,
            &pick(
                &self.language,
                format!("LLM 워커 요청 전송 실패: {detail}"),
                format!("Failed to send request to the LLM worker: {detail}"),
                format!("LLM 工作线程请求发送失败：{detail}"),
            ),
        )
    }

    fn recv_failed_error(&self, detail: &str) -> LlmError {
        LlmError::infer(
            &self.language,
            &pick(
                &self.language,
                format!("LLM 워커 응답 수신 실패: {detail}"),
                format!("Failed to receive response from the LLM worker: {detail}"),
                format!("LLM 工作线程响应接收失败：{detail}"),
            ),
        )
    }

    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>, LlmError> {
        let (respond_to, response) = mpsc::channel();
        self.sender
            .send(WorkerCommand::Embed {
                text: text.to_string(),
                respond_to,
            })
            .map_err(|e| self.send_failed_error(&e.to_string()))?;

        response.recv().map_err(|e| self.recv_failed_error(&e.to_string()))?
    }

    pub fn count_tokens(&self, text: &str) -> Result<usize, LlmError> {
        let (respond_to, response) = mpsc::channel();
        self.sender
            .send(WorkerCommand::CountTokens {
                text: text.to_string(),
                respond_to,
            })
            .map_err(|e| self.send_failed_error(&e.to_string()))?;

        response.recv().map_err(|e| self.recv_failed_error(&e.to_string()))?
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

    pub fn session_statuses(&self) -> Vec<ChatSessionStatus> {
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
