use std::path::PathBuf;

pub const BASE_MODEL_REPO: &str = "Qwen/Qwen2.5-3B-Instruct";

pub struct BaseModelFiles {
    pub safetensors: Vec<PathBuf>,
    pub tokenizer: PathBuf,
}

pub fn ensure_base_model_files() -> anyhow::Result<BaseModelFiles> {
    let api = hf_hub::api::sync::Api::new()?;
    let repo = api.model(BASE_MODEL_REPO.to_string());

    let index_path = repo
        .get("model.safetensors.index.json")
        .map_err(|e| anyhow::anyhow!("safetensors 인덱스 다운로드 실패: {e}"))?;
    let index_bytes = std::fs::read(&index_path)?;
    let index: serde_json::Value = serde_json::from_slice(&index_bytes)?;

    let mut shard_names: Vec<String> = index["weight_map"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("safetensors 인덱스 형식이 예상과 다릅니다"))?
        .values()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    shard_names.sort();
    shard_names.dedup();

    let mut safetensors = Vec::with_capacity(shard_names.len());
    for shard in shard_names {
        let path = repo
            .get(&shard)
            .map_err(|e| anyhow::anyhow!("safetensors 샤드({shard}) 다운로드 실패: {e}"))?;
        safetensors.push(path);
    }

    let tokenizer = repo
        .get("tokenizer.json")
        .map_err(|e| anyhow::anyhow!("토크나이저 다운로드 실패: {e}"))?;

    Ok(BaseModelFiles {
        safetensors,
        tokenizer,
    })
}
