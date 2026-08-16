use std::collections::HashMap;

use candle_core::quantized::gguf_file::Value;

/// google/gemma-2-2b-it 아키텍처 설정. 이 프로젝트가 유일하게 지원하는 로컬 모델
/// (`ai/model/gemma-2-2b-it-Q4_K_M.gguf`)의 실제 GGUF 메타데이터에서 직접 읽어
/// 구성한다(`from_gguf_metadata`) — 하드코딩 추측이 실제 배포된 파일과 어긋나는
/// 사고를 막기 위함이다. head_dim은 hidden_size/heads로 나누어떨어지지 않는
/// 고정값(256)이라 별도 필드로 가진다.
#[derive(Debug, Clone)]
pub struct Gemma2Config {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub head_dim: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f32,
    pub max_position_embeddings: usize,
    pub query_pre_attn_scalar: f64,
    pub attn_logit_softcapping: f64,
    pub final_logit_softcapping: f64,
}

impl Gemma2Config {
    pub fn from_gguf_metadata(metadata: &HashMap<String, Value>) -> anyhow::Result<Self> {
        let get_u32 = |key: &str| -> anyhow::Result<usize> {
            metadata
                .get(key)
                .ok_or_else(|| anyhow::anyhow!("GGUF 메타데이터 키 누락: {key}"))
                .and_then(|v| v.to_u32().map_err(|e| anyhow::anyhow!("{key}: {e}")))
                .map(|v| v as usize)
        };
        let get_f32 = |key: &str, default: f32| -> f32 {
            metadata
                .get(key)
                .and_then(|v| v.to_f32().ok())
                .unwrap_or(default)
        };
        let get_f64 = |key: &str, default: f64| -> f64 {
            metadata
                .get(key)
                .and_then(|v| {
                    v.to_f32()
                        .map(|f| f as f64)
                        .or_else(|_| v.to_f64())
                        .ok()
                })
                .unwrap_or(default)
        };

        let head_count = get_u32("gemma2.attention.head_count")?;
        let head_count_kv = get_u32("gemma2.attention.head_count_kv")?;
        let hidden_size = get_u32("gemma2.embedding_length")?;
        let head_dim = metadata
            .get("gemma2.attention.key_length")
            .and_then(|v| v.to_u32().ok())
            .map(|v| v as usize)
            .unwrap_or(hidden_size / head_count);

        let vocab_size = metadata
            .get("tokenizer.ggml.tokens")
            .and_then(|v| v.to_vec().ok())
            .map(|v| v.len())
            .ok_or_else(|| anyhow::anyhow!("GGUF에 tokenizer.ggml.tokens 배열이 없습니다"))?;

        Ok(Self {
            vocab_size,
            hidden_size,
            intermediate_size: get_u32("gemma2.feed_forward_length")?,
            num_hidden_layers: get_u32("gemma2.block_count")?,
            num_attention_heads: head_count,
            num_key_value_heads: head_count_kv,
            head_dim,
            rms_norm_eps: get_f64("gemma2.attention.layer_norm_rms_epsilon", 1e-6),
            rope_theta: get_f32("gemma2.rope.freq_base", 10_000.0),
            max_position_embeddings: get_u32("gemma2.context_length").unwrap_or(4096),
            // 2B/9B 공식 스펙: query_pre_attn_scalar는 head_dim과 동일하다.
            query_pre_attn_scalar: get_f64("gemma2.attention.query_pre_attn_scalar", head_dim as f64),
            attn_logit_softcapping: get_f64("gemma2.attn_logit_softcapping", 50.0),
            final_logit_softcapping: get_f64("gemma2.final_logit_softcapping", 30.0),
        })
    }
}
