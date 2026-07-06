use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;

use super::{CacheGenerationResult, LlmError};

#[derive(Debug, Clone, Serialize)]
pub struct LlmRequestStatus {
    pub request_id: String,
    pub persona_id: Option<String>,
    pub state: String,
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub reused_prefix_tokens: usize,
    pub truncated_prompt_tokens: usize,
    pub cache_reset: bool,
    pub error_message: Option<String>,
}

#[derive(Clone)]
pub struct RequestRegistry {
    requests: Arc<Mutex<HashMap<String, LlmRequestStatus>>>,
    cancellations: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

impl RequestRegistry {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            cancellations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, request_id: &str, persona_id: Option<String>) -> Arc<AtomicBool> {
        let cancel_flag = Arc::new(AtomicBool::new(false));
        if let Ok(mut map) = self.cancellations.lock() {
            map.insert(request_id.to_string(), Arc::clone(&cancel_flag));
        }
        if let Ok(mut map) = self.requests.lock() {
            map.insert(
                request_id.to_string(),
                LlmRequestStatus {
                    request_id: request_id.to_string(),
                    persona_id,
                    state: "running".to_string(),
                    prompt_tokens: 0,
                    generated_tokens: 0,
                    reused_prefix_tokens: 0,
                    truncated_prompt_tokens: 0,
                    cache_reset: false,
                    error_message: None,
                },
            );
        }
        cancel_flag
    }

    pub fn update_generation(&self, request_id: &str, result: &CacheGenerationResult) {
        if let Ok(mut map) = self.requests.lock() {
            if let Some(status) = map.get_mut(request_id) {
                status.prompt_tokens = result.prompt_tokens;
                status.generated_tokens = result.generated_tokens;
                status.reused_prefix_tokens = result.reused_prefix_tokens;
                status.truncated_prompt_tokens = result.truncated_prompt_tokens;
                status.cache_reset = result.cache_reset;
            }
        }
    }

    pub fn finish(&self, request_id: &str, result: &Result<String, LlmError>, cancelled: bool) {
        if let Ok(mut map) = self.requests.lock() {
            if let Some(status) = map.get_mut(request_id) {
                status.state = if cancelled {
                    "cancelled".to_string()
                } else if result.is_ok() {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                };
                status.error_message = result.as_ref().err().map(ToString::to_string);
            }
        }
        if let Ok(mut map) = self.cancellations.lock() {
            map.remove(request_id);
        }
    }

    pub fn cancel(&self, request_id: &str) -> bool {
        self.cancellations
            .lock()
            .ok()
            .and_then(|map| map.get(request_id).cloned())
            .is_some_and(|flag| {
                flag.store(true, Ordering::SeqCst);
                true
            })
    }

    pub fn statuses(&self) -> Vec<LlmRequestStatus> {
        self.requests
            .lock()
            .map(|map| map.values().cloned().collect())
            .unwrap_or_default()
    }
}
