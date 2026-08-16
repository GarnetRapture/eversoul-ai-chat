use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::LlmError;
use crate::infrastructure::cache::CacheController;
use crate::infrastructure::i18n::pick;
use crate::startup_debug_log;

#[derive(Debug, Clone, Serialize)]
pub struct ModelFileValidation {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub sidecar_sha256: Option<String>,
    pub hash_matches_sidecar: Option<bool>,
}

pub fn validate_model_file(
    path: &Path,
    language: &str,
    cache: &CacheController,
) -> Result<ModelFileValidation, LlmError> {
    if !path.exists() {
        return Err(LlmError::model_file_not_found(language, path));
    }
    if path.extension().and_then(|value| value.to_str()) != Some("gguf") {
        return Err(LlmError::model_load(
            language,
            &pick(
                language,
                format!("GGUF 모델 파일이 아닙니다: {}", path.display()),
                format!("Not a GGUF model file: {}", path.display()),
                format!("不是 GGUF 模型文件：{}", path.display()),
            ),
        ));
    }

    let cache_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let metadata = path
        .metadata()
        .map_err(|e| LlmError::model_load(language, &e.to_string()))?;
    if metadata.len() == 0 {
        return Err(LlmError::model_load(
            language,
            &pick(
                language,
                format!("모델 파일이 비어 있습니다: {}", path.display()),
                format!("The model file is empty: {}", path.display()),
                format!("模型文件为空：{}", path.display()),
            ),
        ));
    }

    if let Some(cached) = cache.cached_model_validation(&cache_key, &metadata) {
        startup_debug_log("llm_validation:cache_hit");
        return Ok(cached);
    }

    startup_debug_log("llm_validation:hash:start");
    let mut file = File::open(path).map_err(|e| LlmError::model_load(language, &e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| LlmError::model_load(language, &e.to_string()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let sha256 = hex::encode(hasher.finalize());
    startup_debug_log("llm_validation:hash:done");
    let sidecar_sha256 = read_sidecar_hash(path, language)?;
    let hash_matches_sidecar = sidecar_sha256
        .as_ref()
        .map(|expected| expected.eq_ignore_ascii_case(&sha256));

    if hash_matches_sidecar == Some(false) {
        return Err(LlmError::model_load(
            language,
            &pick(
                language,
                format!("모델 SHA-256 불일치: {}", path.display()),
                format!("Model SHA-256 mismatch: {}", path.display()),
                format!("模型 SHA-256 不匹配：{}", path.display()),
            ),
        ));
    }

    let validation = ModelFileValidation {
        path: path.to_string_lossy().to_string(),
        size_bytes: metadata.len(),
        sha256,
        sidecar_sha256,
        hash_matches_sidecar,
    };
    cache.store_model_validation(cache_key, &metadata, validation.clone());
    startup_debug_log("llm_validation:cache_store");
    Ok(validation)
}

fn read_sidecar_hash(path: &Path, language: &str) -> Result<Option<String>, LlmError> {
    let candidates = [
        PathBuf::from(format!("{}.sha256", path.to_string_lossy())),
        path.with_extension("gguf.sha256"),
    ];
    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let file =
            File::open(&candidate).map_err(|e| LlmError::model_load(language, &e.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| LlmError::model_load(language, &e.to_string()))?;
        let hash = line
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if hash.len() == 64 && hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Ok(Some(hash));
        }
        return Err(LlmError::model_load(
            language,
            &pick(
                language,
                format!("SHA-256 사이드카 형식이 올바르지 않습니다: {}", candidate.display()),
                format!("Invalid SHA-256 sidecar format: {}", candidate.display()),
                format!("SHA-256 附属文件格式无效：{}", candidate.display()),
            ),
        ));
    }
    Ok(None)
}
