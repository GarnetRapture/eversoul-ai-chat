use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ExternalAiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Clone, Serialize)]
struct ChatCompletionMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionResponseMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponseMessage {
    content: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExternalAiError {
    #[error("external API key is not configured")]
    MissingApiKey,
    #[error("external API request failed: {0}")]
    Request(String),
    #[error("external API returned {status}: {body}")]
    Response { status: StatusCode, body: String },
    #[error("external API returned an empty response")]
    EmptyResponse,
}

pub async fn infer_chat(
    config: &ExternalAiConfig,
    system_prompt: &str,
    history: &[crate::domains::chat::types::ChatMessage],
    max_tokens: u32,
) -> Result<String, ExternalAiError> {
    if config.api_key.trim().is_empty() {
        return Err(ExternalAiError::MissingApiKey);
    }

    let mut messages = vec![ChatCompletionMessage {
        role: "system".to_string(),
        content: system_prompt.to_string(),
    }];
    messages.extend(history.iter().map(|message| ChatCompletionMessage {
        role: normalize_role(&message.role),
        content: message.content.clone(),
    }));

    let endpoint = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let payload = ChatCompletionRequest {
        model: config.model.clone(),
        messages,
        max_tokens,
        temperature: 0.8,
    };

    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|err| ExternalAiError::Request(err.to_string()))?
        .post(endpoint)
        .bearer_auth(config.api_key.trim())
        .json(&payload)
        .send()
        .await
        .map_err(|err| ExternalAiError::Request(err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|err| format!("failed to read error body: {err}"));
        return Err(ExternalAiError::Response { status, body });
    }

    let completion = response
        .json::<ChatCompletionResponse>()
        .await
        .map_err(|err| ExternalAiError::Request(err.to_string()))?;

    completion
        .choices
        .into_iter()
        .find_map(|choice| choice.message.content)
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or(ExternalAiError::EmptyResponse)
}

fn normalize_role(role: &str) -> String {
    match role {
        "user" | "assistant" | "system" => role.to_string(),
        _ => "user".to_string(),
    }
}
