use ini::Ini;
use std::path::{Path, PathBuf};

const SECTION_GENERAL: &str = "general";
const KEY_DEFAULT_PERSONA_ID: &str = "default_persona_id";
const KEY_ACTIVE_STYLE_ID: &str = "active_style_id";

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

    pub fn reset(&self) -> std::io::Result<()> {
        self.persist(&Ini::new())
    }
}
