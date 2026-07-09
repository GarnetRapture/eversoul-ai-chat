use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::Serialize;
use thiserror::Error;

pub const QWEN_MODEL_DOWNLOAD_URL: &str =
    "https://huggingface.co/MyeongHo0621/Qwen2.5-3B-Korean/resolve/main/gguf/qwen25-3b-korean-Q4_K_M.gguf";
pub const GEMMA_MODEL_DOWNLOAD_URL: &str =
    "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf";

pub fn get_model_download_url(active_model: &str) -> &'static str {
    if active_model.starts_with("gemma") {
        GEMMA_MODEL_DOWNLOAD_URL
    } else {
        QWEN_MODEL_DOWNLOAD_URL
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub ratio: f64,
    pub done: bool,
}

impl ModelDownloadProgress {
    fn new(downloaded_bytes: u64, total_bytes: u64, done: bool) -> Self {
        let ratio = if total_bytes > 0 {
            (downloaded_bytes as f64 / total_bytes as f64).clamp(0.0, 1.0)
        } else if done {
            1.0
        } else {
            0.0
        };
        Self {
            downloaded_bytes,
            total_bytes,
            ratio,
            done,
        }
    }
}

#[derive(Debug, Error)]
pub enum ModelDownloadError {
    #[error("모델 다운로드 요청 실패: {0}")]
    Request(String),

    #[error("모델 다운로드 파일 기록 실패: {0}")]
    Io(String),
}

fn partial_path(dest_path: &Path) -> PathBuf {
    let mut file_name = dest_path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    file_name.push(".part");
    dest_path.with_file_name(file_name)
}

pub async fn download_model_file<F>(
    dest_path: &Path,
    active_model: &str,
    mut on_progress: F,
) -> Result<(), ModelDownloadError>
where
    F: FnMut(ModelDownloadProgress),
{
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ModelDownloadError::Io(e.to_string()))?;
    }

    let client = reqwest::Client::new();
    let url = get_model_download_url(active_model);
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| ModelDownloadError::Request(e.to_string()))?
        .error_for_status()
        .map_err(|e| ModelDownloadError::Request(e.to_string()))?;

    let total_bytes = response.content_length().unwrap_or(0);
    on_progress(ModelDownloadProgress::new(0, total_bytes, false));

    let tmp_path = partial_path(dest_path);
    let mut file =
        std::fs::File::create(&tmp_path).map_err(|e| ModelDownloadError::Io(e.to_string()))?;

    let mut downloaded_bytes: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => {
                drop(file);
                let _ = std::fs::remove_file(&tmp_path);
                return Err(ModelDownloadError::Request(e.to_string()));
            }
        };
        if let Err(e) = file.write_all(&chunk) {
            drop(file);
            let _ = std::fs::remove_file(&tmp_path);
            return Err(ModelDownloadError::Io(e.to_string()));
        }
        downloaded_bytes += chunk.len() as u64;
        on_progress(ModelDownloadProgress::new(downloaded_bytes, total_bytes, false));
    }

    file.flush()
        .map_err(|e| ModelDownloadError::Io(e.to_string()))?;
    drop(file);

    std::fs::rename(&tmp_path, dest_path).map_err(|e| ModelDownloadError::Io(e.to_string()))?;

    let final_total = total_bytes.max(downloaded_bytes);
    on_progress(ModelDownloadProgress::new(downloaded_bytes, final_total, true));
    Ok(())
}
