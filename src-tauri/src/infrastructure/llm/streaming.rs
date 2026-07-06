use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::LlmError;

#[derive(Debug, Clone, Serialize)]
pub struct StreamTokenEvent {
    pub request_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamDoneEvent {
    pub request_id: String,
    pub cancelled: bool,
    pub error_message: Option<String>,
}

#[derive(Clone)]
pub struct StreamTarget {
    app_handle: AppHandle,
    token_event: String,
    done_event: String,
}

impl StreamTarget {
    pub fn new(app_handle: AppHandle, token_event: String, done_event: String) -> Self {
        Self {
            app_handle,
            token_event,
            done_event,
        }
    }

    pub fn emit_token(&self, request_id: &str, token: String) -> Result<(), LlmError> {
        self.app_handle
            .emit(
                &self.token_event,
                StreamTokenEvent {
                    request_id: request_id.to_string(),
                    token,
                },
            )
            .map_err(|e| LlmError::Infer(e.to_string()))
    }

    pub fn emit_done(&self, request_id: &str, cancelled: bool, error_message: Option<String>) {
        let _ = self.app_handle.emit(
            &self.done_event,
            StreamDoneEvent {
                request_id: request_id.to_string(),
                cancelled,
                error_message,
            },
        );
    }
}
