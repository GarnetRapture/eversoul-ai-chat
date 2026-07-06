use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::LlmError;
use crate::startup_debug_log;

#[derive(Debug, Clone, Serialize)]
pub struct ModelFileValidation {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub sidecar_sha256: Option<String>,
    pub hash_matches_sidecar: Option<bool>,
}

#[derive(Debug, Clone)]
struct ModelValidationCacheEntry {
    size_bytes: u64,
    modified_at: Option<SystemTime>,
    validation: ModelFileValidation,
}

static MODEL_VALIDATION_CACHE: OnceLock<Mutex<HashMap<PathBuf, ModelValidationCacheEntry>>> =
    OnceLock::new();

pub fn validate_model_file(path: &Path) -> Result<ModelFileValidation, LlmError> {
    if !path.exists() {
        return Err(LlmError::ModelFileNotFound {
            path: path.to_path_buf(),
        });
    }
    if path.extension().and_then(|value| value.to_str()) != Some("gguf") {
        return Err(LlmError::ModelLoad(format!(
            "GGUF 모델 파일이 아닙니다: {}",
            path.display()
        )));
    }

    let cache_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let metadata = path
        .metadata()
        .map_err(|e| LlmError::ModelLoad(e.to_string()))?;
    if metadata.len() == 0 {
        return Err(LlmError::ModelLoad(format!(
            "모델 파일이 비어 있습니다: {}",
            path.display()
        )));
    }

    if let Some(cached) = cached_validation(&cache_key, &metadata)? {
        startup_debug_log("llm_validation:cache_hit");
        return Ok(cached);
    }

    startup_debug_log("llm_validation:hash:start");
    let mut file = File::open(path).map_err(|e| LlmError::ModelLoad(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let sha256 = hex::encode(hasher.finalize());
    startup_debug_log("llm_validation:hash:done");
    let sidecar_sha256 = read_sidecar_hash(path)?;
    let hash_matches_sidecar = sidecar_sha256
        .as_ref()
        .map(|expected| expected.eq_ignore_ascii_case(&sha256));

    if hash_matches_sidecar == Some(false) {
        return Err(LlmError::ModelLoad(format!(
            "모델 SHA-256 불일치: {}",
            path.display()
        )));
    }

    let validation = ModelFileValidation {
        path: path.to_string_lossy().to_string(),
        size_bytes: metadata.len(),
        sha256,
        sidecar_sha256,
        hash_matches_sidecar,
    };
    store_cached_validation(cache_key, metadata, validation.clone())?;
    Ok(validation)
}

fn cached_validation(
    cache_key: &Path,
    metadata: &Metadata,
) -> Result<Option<ModelFileValidation>, LlmError> {
    let modified_at = metadata.modified().ok();
    let cache = MODEL_VALIDATION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let guard = cache
        .lock()
        .map_err(|e| LlmError::ModelLoad(e.to_string()))?;
    Ok(guard.get(cache_key).and_then(|entry| {
        (entry.size_bytes == metadata.len() && entry.modified_at == modified_at)
            .then(|| entry.validation.clone())
    }))
}

fn store_cached_validation(
    cache_key: PathBuf,
    metadata: Metadata,
    validation: ModelFileValidation,
) -> Result<(), LlmError> {
    let entry = ModelValidationCacheEntry {
        size_bytes: metadata.len(),
        modified_at: metadata.modified().ok(),
        validation,
    };
    let cache = MODEL_VALIDATION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    cache
        .lock()
        .map_err(|e| LlmError::ModelLoad(e.to_string()))?
        .insert(cache_key, entry);
    startup_debug_log("llm_validation:cache_store");
    Ok(())
}

fn read_sidecar_hash(path: &Path) -> Result<Option<String>, LlmError> {
    let candidates = [
        PathBuf::from(format!("{}.sha256", path.to_string_lossy())),
        path.with_extension("gguf.sha256"),
    ];
    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let file = File::open(&candidate).map_err(|e| LlmError::ModelLoad(e.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;
        let hash = line
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if hash.len() == 64 && hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Ok(Some(hash));
        }
        return Err(LlmError::ModelLoad(format!(
            "SHA-256 사이드카 형식이 올바르지 않습니다: {}",
            candidate.display()
        )));
    }
    Ok(None)
}
