use crate::infrastructure::external_ai::{ExternalAiConfig, ExternalAiError};
use ini::Ini;
use std::path::{Path, PathBuf};

const SECTION_API: &str = "api";
const KEY_PROVIDER: &str = "provider";
const KEY_LOCAL_API_KEY: &str = "local_api_key";
const KEY_EXTERNAL_ENABLED: &str = "external_enabled";
const KEY_EXTERNAL_BASE_URL: &str = "external_base_url";
const KEY_EXTERNAL_API_KEY: &str = "external_api_key";
const KEY_EXTERNAL_MODEL: &str = "external_model";
const DEFAULT_EXTERNAL_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_EXTERNAL_MODEL: &str = "gpt-4o-mini";
pub const CONNECTION_TEST_MAX_TOKENS: u32 = 32;

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyError {
    #[error("io error: {0}")]
    Io(String),
    #[error("external_api_base_url must start with https:// or http://")]
    InvalidBaseUrl,
    #[error("external_api_model is required when external API is enabled")]
    MissingModel,
    #[error("external_api_key is required when external API is enabled")]
    MissingApiKey,
    #[error("external api call failed: {0}")]
    ExternalCall(#[from] ExternalAiError),
}

/// API 제공자/키의 저장·검증·외부 API 호출·로컬-외부 분기 판단을 한 곳에서 처리하는
/// 컨트롤러. domains::settings::services가 이 컨트롤러를 호출해 SettingsError로
/// 감싸고, domains::chat::commands가 resolve_chat_backend로 로컬/외부 분기를 판단한다.
pub struct ApiKeyController {
    ini_path: PathBuf,
}

impl ApiKeyController {
    pub fn new<P: AsRef<Path>>(ini_path: P) -> Self {
        Self {
            ini_path: ini_path.as_ref().to_path_buf(),
        }
    }

    fn load(&self) -> Ini {
        Ini::load_from_file(&self.ini_path).unwrap_or_else(|_| Ini::new())
    }

    fn persist(&self, conf: &Ini) -> std::io::Result<()> {
        if let Some(parent) = self.ini_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        conf.write_to_file(&self.ini_path)
    }

    pub fn get_provider(&self) -> Option<String> {
        self.load()
            .get_from(Some(SECTION_API), KEY_PROVIDER)
            .map(|s| s.to_string())
    }

    pub fn set_provider(&self, provider: Option<&str>) -> std::io::Result<()> {
        let mut conf = self.load();
        if let Some(p) = provider {
            conf.with_section(Some(SECTION_API)).set(KEY_PROVIDER, p);
        } else {
            conf.with_section(Some(SECTION_API)).delete(&KEY_PROVIDER);
        }
        self.persist(&conf)
    }

    pub fn get_local_api_key(&self) -> Option<String> {
        self.load()
            .get_from(Some(SECTION_API), KEY_LOCAL_API_KEY)
            .map(|s| s.to_string())
    }

    pub fn set_local_api_key(&self, key: Option<&str>) -> std::io::Result<()> {
        let mut conf = self.load();
        if let Some(k) = key {
            conf.with_section(Some(SECTION_API)).set(KEY_LOCAL_API_KEY, k);
        } else {
            conf.with_section(Some(SECTION_API)).delete(&KEY_LOCAL_API_KEY);
        }
        self.persist(&conf)
    }

    pub fn get_external_enabled(&self) -> bool {
        self.load()
            .get_from(Some(SECTION_API), KEY_EXTERNAL_ENABLED)
            .is_some_and(|v| v == "true")
    }

    pub fn get_external_base_url(&self) -> String {
        self.load()
            .get_from(Some(SECTION_API), KEY_EXTERNAL_BASE_URL)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(DEFAULT_EXTERNAL_BASE_URL)
            .trim_end_matches('/')
            .to_string()
    }

    pub fn get_external_api_key(&self) -> Option<String> {
        self.load()
            .get_from(Some(SECTION_API), KEY_EXTERNAL_API_KEY)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    }

    pub fn get_external_model(&self) -> String {
        self.load()
            .get_from(Some(SECTION_API), KEY_EXTERNAL_MODEL)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(DEFAULT_EXTERNAL_MODEL)
            .to_string()
    }

    fn store_external_config(
        &self,
        enabled: bool,
        base_url: &str,
        api_key: &str,
        model: &str,
    ) -> std::io::Result<()> {
        let mut conf = self.load();
        conf.with_section(Some(SECTION_API))
            .set(KEY_EXTERNAL_ENABLED, if enabled { "true" } else { "false" })
            .set(KEY_EXTERNAL_BASE_URL, base_url)
            .set(KEY_EXTERNAL_API_KEY, api_key)
            .set(KEY_EXTERNAL_MODEL, model);
        self.persist(&conf)
    }

    fn normalize_base_url(base_url: &str) -> Result<String, ApiKeyError> {
        let mut normalized = base_url.trim().trim_end_matches('/').to_string();
        if normalized.ends_with("/chat/completions") {
            normalized.truncate(normalized.len() - "/chat/completions".len());
            normalized = normalized.trim_end_matches('/').to_string();
        }
        if normalized.is_empty() {
            normalized = DEFAULT_EXTERNAL_BASE_URL.to_string();
        }
        if !normalized.starts_with("https://") && !normalized.starts_with("http://") {
            return Err(ApiKeyError::InvalidBaseUrl);
        }
        Ok(normalized)
    }

    /// 검증(base_url 정규화·형식 검사, enabled일 때 model/key 필수)까지 마친 뒤 저장한다.
    /// api_key가 비어 있으면 기존에 저장된 값을 그대로 유지한다.
    pub fn apply_external_config(
        &self,
        enabled: bool,
        base_url: &str,
        api_key: &str,
        model: &str,
    ) -> Result<(), ApiKeyError> {
        let normalized_base_url = Self::normalize_base_url(base_url)?;
        let normalized_model = model.trim();
        if enabled && normalized_model.is_empty() {
            return Err(ApiKeyError::MissingModel);
        }

        let trimmed_key = api_key.trim();
        let resolved_key = if trimmed_key.is_empty() {
            self.get_external_api_key().unwrap_or_default()
        } else {
            trimmed_key.to_string()
        };
        if enabled && resolved_key.is_empty() {
            return Err(ApiKeyError::MissingApiKey);
        }

        let resolved_model = if normalized_model.is_empty() {
            DEFAULT_EXTERNAL_MODEL
        } else {
            normalized_model
        };

        self.store_external_config(enabled, &normalized_base_url, &resolved_key, resolved_model)
            .map_err(|e| ApiKeyError::Io(e.to_string()))
    }

/// 저장된 값 그대로 ExternalAiConfig를 조합한다(enabled 여부와 무관). 연결 테스트용.
    pub fn build_external_config(&self) -> Option<ExternalAiConfig> {
        let api_key = self.get_external_api_key()?;
        Some(ExternalAiConfig {
            base_url: self.get_external_base_url(),
            api_key,
            model: self.get_external_model(),
        })
    }

    /// 활성화된 외부 API 설정이 있으면 ExternalAiConfig를 반환하고, 아니면 None.
    /// chat_send_message가 이 값의 Some/None만으로 로컬/외부 백엔드를 분기한다.
    pub fn resolve_chat_backend(&self) -> Option<ExternalAiConfig> {
        if !self.get_external_enabled() {
            return None;
        }
        self.build_external_config()
    }

    pub fn reset(&self) -> std::io::Result<()> {
        self.persist(&Ini::new())
    }
}
