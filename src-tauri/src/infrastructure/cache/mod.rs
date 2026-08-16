use std::collections::HashMap;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use crate::infrastructure::llm::validation::ModelFileValidation;

#[derive(Debug, Clone)]
struct ModelValidationCacheEntry {
    size_bytes: u64,
    modified_at: Option<SystemTime>,
    validation: ModelFileValidation,
}

pub struct CacheController {
    session_cache_dir: PathBuf,
    model_validation_cache: Mutex<HashMap<PathBuf, ModelValidationCacheEntry>>,
}

impl CacheController {
    pub fn new(session_cache_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&session_cache_dir)?;
        Ok(Self {
            session_cache_dir,
            model_validation_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn default_session_cache_dir() -> PathBuf {
        let mut dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        dir.push("ai");
        dir.push("cache");
        dir
    }

    pub fn session_cache_dir(&self) -> &Path {
        &self.session_cache_dir
    }

    pub fn session_cache_path(&self, persona_id: &str) -> PathBuf {
        self.session_cache_dir.join(format!("{persona_id}.bin"))
    }

    pub fn cached_model_validation(
        &self,
        cache_key: &Path,
        metadata: &Metadata,
    ) -> Option<ModelFileValidation> {
        let modified_at = metadata.modified().ok();
        let guard = self.model_validation_cache.lock().ok()?;
        guard.get(cache_key).and_then(|entry| {
            (entry.size_bytes == metadata.len() && entry.modified_at == modified_at)
                .then(|| entry.validation.clone())
        })
    }

    pub fn store_model_validation(
        &self,
        cache_key: PathBuf,
        metadata: &Metadata,
        validation: ModelFileValidation,
    ) {
        let entry = ModelValidationCacheEntry {
            size_bytes: metadata.len(),
            modified_at: metadata.modified().ok(),
            validation,
        };
        if let Ok(mut guard) = self.model_validation_cache.lock() {
            guard.insert(cache_key, entry);
        }
    }
}
