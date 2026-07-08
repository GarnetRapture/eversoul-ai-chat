use ini::Ini;
use std::path::{Path, PathBuf};

const SECTION_GENERAL: &str = "general";
const KEY_DEFAULT_PERSONA_ID: &str = "default_persona_id";
const KEY_ACTIVE_STYLE_ID: &str = "active_style_id";
const KEY_LANGUAGE: &str = "language";
const KEY_PERFORMANCE_TIER: &str = "performance_tier";
const KEY_SETUP_STAGE: &str = "setup_stage";
const KEY_SHOW_REASONING: &str = "show_reasoning";
const KEY_EXTERNAL_API_ENABLED: &str = "external_api_enabled";
const KEY_EXTERNAL_API_BASE_URL: &str = "external_api_base_url";
const KEY_EXTERNAL_API_KEY: &str = "external_api_key";
const KEY_EXTERNAL_API_MODEL: &str = "external_api_model";
const DEFAULT_LANGUAGE: &str = "ko";
const DEFAULT_PERFORMANCE_TIER: &str = "balanced";
const DEFAULT_EXTERNAL_API_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_EXTERNAL_API_MODEL: &str = "gpt-4o-mini";
const SUPPORTED_PERFORMANCE_TIERS: [&str; 3] = ["light", "balanced", "performance"];

pub const SETUP_STAGE_LANGUAGE: &str = "language";
pub const SETUP_STAGE_DOWNLOAD: &str = "download";
pub const SETUP_STAGE_PERFORMANCE: &str = "performance";
pub const SETUP_STAGE_DONE: &str = "done";
const SUPPORTED_SETUP_STAGES: [&str; 4] = [
    SETUP_STAGE_LANGUAGE,
    SETUP_STAGE_DOWNLOAD,
    SETUP_STAGE_PERFORMANCE,
    SETUP_STAGE_DONE,
];

pub struct SettingsManager {
    ini_path: PathBuf,
}

impl SettingsManager {
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

    pub fn modules_path(&self) -> PathBuf {
        self.ini_path
            .parent()
            .map(|parent| parent.join("modules.json"))
            .unwrap_or_else(|| PathBuf::from("modules.json"))
    }

    pub fn get_default_persona_id(&self) -> Option<String> {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_DEFAULT_PERSONA_ID)
            .map(|s| s.to_string())
    }

    pub fn set_default_persona_id(&self, id: &str) -> std::io::Result<()> {
        let mut conf = self.load();
        conf.with_section(Some(SECTION_GENERAL))
            .set(KEY_DEFAULT_PERSONA_ID, id);
        self.persist(&conf)
    }

    pub fn get_active_style_id(&self) -> Option<String> {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_ACTIVE_STYLE_ID)
            .map(|s| s.to_string())
    }

    pub fn set_active_style_id(&self, id: &str) -> std::io::Result<()> {
        let mut conf = self.load();
        conf.with_section(Some(SECTION_GENERAL))
            .set(KEY_ACTIVE_STYLE_ID, id);
        self.persist(&conf)
    }

    pub fn get_language(&self) -> String {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_LANGUAGE)
            .filter(|value| matches!(*value, "ko" | "en" | "zh_cn"))
            .unwrap_or(DEFAULT_LANGUAGE)
            .to_string()
    }

    pub fn has_language(&self) -> bool {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_LANGUAGE)
            .is_some_and(|value| matches!(value, "ko" | "en" | "zh_cn"))
    }

    pub fn set_language(&self, language: &str) -> std::io::Result<()> {
        let mut conf = self.load();
        let normalized = if matches!(language, "ko" | "en" | "zh_cn") {
            language
        } else {
            DEFAULT_LANGUAGE
        };
        conf.with_section(Some(SECTION_GENERAL))
            .set(KEY_LANGUAGE, normalized);
        self.persist(&conf)
    }

    pub fn get_performance_tier(&self) -> String {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_PERFORMANCE_TIER)
            .filter(|value| SUPPORTED_PERFORMANCE_TIERS.contains(value))
            .unwrap_or(DEFAULT_PERFORMANCE_TIER)
            .to_string()
    }

    pub fn has_performance_tier(&self) -> bool {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_PERFORMANCE_TIER)
            .is_some_and(|value| SUPPORTED_PERFORMANCE_TIERS.contains(&value))
    }

    pub fn set_performance_tier(&self, tier: &str) -> std::io::Result<()> {
        let mut conf = self.load();
        let normalized = if SUPPORTED_PERFORMANCE_TIERS.contains(&tier) {
            tier
        } else {
            DEFAULT_PERFORMANCE_TIER
        };
        conf.with_section(Some(SECTION_GENERAL))
            .set(KEY_PERFORMANCE_TIER, normalized);
        self.persist(&conf)
    }

    pub fn get_setup_stage(&self) -> String {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_SETUP_STAGE)
            .filter(|value| SUPPORTED_SETUP_STAGES.contains(value))
            .unwrap_or(SETUP_STAGE_LANGUAGE)
            .to_string()
    }

    pub fn set_setup_stage(&self, stage: &str) -> std::io::Result<()> {
        let mut conf = self.load();
        let normalized = if SUPPORTED_SETUP_STAGES.contains(&stage) {
            stage
        } else {
            SETUP_STAGE_LANGUAGE
        };
        conf.with_section(Some(SECTION_GENERAL))
            .set(KEY_SETUP_STAGE, normalized);
        self.persist(&conf)
    }

    pub fn get_show_reasoning(&self) -> bool {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_SHOW_REASONING)
            .map_or(true, |v| v == "true") // 기본값 true
    }

    pub fn set_show_reasoning(&self, value: bool) -> std::io::Result<()> {
        let mut conf = self.load();
        conf.with_section(Some(SECTION_GENERAL))
            .set(KEY_SHOW_REASONING, if value { "true" } else { "false" });
        self.persist(&conf)
    }

    pub fn get_external_api_enabled(&self) -> bool {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_EXTERNAL_API_ENABLED)
            .is_some_and(|v| v == "true")
    }

    pub fn get_external_api_base_url(&self) -> String {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_EXTERNAL_API_BASE_URL)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(DEFAULT_EXTERNAL_API_BASE_URL)
            .trim_end_matches('/')
            .to_string()
    }

    pub fn get_external_api_key(&self) -> Option<String> {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_EXTERNAL_API_KEY)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    }

    pub fn get_external_api_model(&self) -> String {
        self.load()
            .get_from(Some(SECTION_GENERAL), KEY_EXTERNAL_API_MODEL)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(DEFAULT_EXTERNAL_API_MODEL)
            .to_string()
    }

    pub fn set_external_api_config(
        &self,
        enabled: bool,
        base_url: &str,
        api_key: &str,
        model: &str,
    ) -> std::io::Result<()> {
        let mut conf = self.load();
        let normalized_base_url = base_url.trim().trim_end_matches('/');
        let normalized_model = model.trim();
        conf.with_section(Some(SECTION_GENERAL))
            .set(
                KEY_EXTERNAL_API_ENABLED,
                if enabled { "true" } else { "false" },
            )
            .set(
                KEY_EXTERNAL_API_BASE_URL,
                if normalized_base_url.is_empty() {
                    DEFAULT_EXTERNAL_API_BASE_URL
                } else {
                    normalized_base_url
                },
            )
            .set(KEY_EXTERNAL_API_KEY, api_key.trim())
            .set(
                KEY_EXTERNAL_API_MODEL,
                if normalized_model.is_empty() {
                    DEFAULT_EXTERNAL_API_MODEL
                } else {
                    normalized_model
                },
            );
        self.persist(&conf)
    }

    pub fn ensure_initialized(&self) -> std::io::Result<()> {
        if self.ini_path.exists() {
            return Ok(());
        }
        let mut conf = self.load();
        conf.with_section(Some(SECTION_GENERAL))
            .set(KEY_SETUP_STAGE, SETUP_STAGE_LANGUAGE);
        self.persist(&conf)
    }

    pub fn reset(&self) -> std::io::Result<()> {
        self.persist(&Ini::new())
    }
}
