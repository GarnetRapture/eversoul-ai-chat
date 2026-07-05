use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use llama_cpp_2::{
    context::params::{LlamaContextParams, LlamaPoolingType},
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, LlamaLoraAdapter, LlamaModel},
    sampling::LlamaSampler,
    TokenToStringError,
};
use thiserror::Error;

pub const MODEL_RELATIVE_PATH: &str = "ai/model/qwen25-3b-korean-Q4_K_M.gguf";

const CONTEXT_SIZE: u32 = 8192;

const DEFAULT_MAX_TOKENS: u32 = 512;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("모델 파일을 찾을 수 없습니다: {path}")]
    ModelFileNotFound { path: PathBuf },

    #[error("llama.cpp 백엔드 초기화 실패: {0}")]
    BackendInit(String),

    #[error("모델 로딩 실패: {0}")]
    ModelLoad(String),

    #[error("컨텍스트 생성 실패: {0}")]
    ContextCreate(String),

    #[error("토큰화 실패: {0}")]
    Tokenize(String),

    #[error("추론 실패: {0}")]
    Infer(String),
}

pub struct LlmEngine {
    backend: LlamaBackend,
    model: LlamaModel,
    model_path: PathBuf,
    adapters_dir: PathBuf,
    lora_adapters: Mutex<HashMap<String, LlamaLoraAdapter>>,
}

impl LlmEngine {
    fn token_to_bytes(&self, token: llama_cpp_2::token::LlamaToken) -> Result<Vec<u8>, LlmError> {
        match self.model.token_to_piece_bytes(token, 8, true, None) {
            Ok(bytes) => Ok(bytes),
            Err(TokenToStringError::InsufficientBufferSpace(size)) if size.is_negative() => self
                .model
                .token_to_piece_bytes(token, (-size) as usize, true, None)
                .map_err(|e| LlmError::Infer(e.to_string())),
            Err(err) => Err(LlmError::Infer(err.to_string())),
        }
    }

    pub fn load(app_root: &Path, adapters_dir: PathBuf) -> Result<Self, LlmError> {
        let model_path = app_root.join(MODEL_RELATIVE_PATH);

        if !model_path.exists() {
            return Err(LlmError::ModelFileNotFound {
                path: model_path.clone(),
            });
        }

        let backend = LlamaBackend::init().map_err(|e| LlmError::BackendInit(e.to_string()))?;

        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;

        Ok(Self {
            backend,
            model,
            model_path,
            adapters_dir,
            lora_adapters: Mutex::new(HashMap::new()),
        })
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn infer(
        &self,
        prompt: &str,
        max_tokens: Option<u32>,
        persona_id: Option<&str>,
    ) -> Result<String, LlmError> {
        let max_tokens = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
        if let Some(id) = persona_id {
            self.mount_persona_adapter(id)?;
        }

        let physical_cores = num_cpus::get_physical() as i32;
        let n_threads = (physical_cores - 1).max(1);

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(CONTEXT_SIZE))
            .with_n_threads(n_threads)
            .with_n_threads_batch(n_threads);

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;

        if let Some(id) = persona_id {
            let mut adapters = self
                .lora_adapters
                .lock()
                .map_err(|e| LlmError::ContextCreate(e.to_string()))?;
            if let Some(adapter) = adapters.get_mut(id) {
                ctx.lora_adapter_set(adapter, 1.0)
                    .map_err(|e| LlmError::ContextCreate(e.to_string()))?;
            }
        }

        let tokens = self
            .model
            .str_to_token(prompt, llama_cpp_2::model::AddBos::Never)
            .map_err(|e| LlmError::Tokenize(e.to_string()))?;

        let n_tokens = tokens.len();
        let mut batch = LlamaBatch::new(n_tokens, 1);
        for (i, &token) in tokens.iter().enumerate() {
            let is_last = i == n_tokens - 1;
            batch
                .add(token, i as i32, &[0], is_last)
                .map_err(|e| LlmError::Infer(e.to_string()))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| LlmError::Infer(e.to_string()))?;

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(256, 1.15, 0.0, 0.0),
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::temp(0.75),
            LlamaSampler::dist(seed),
        ]);
        let mut output_tokens: Vec<llama_cpp_2::token::LlamaToken> = Vec::new();
        let mut n_cur = n_tokens as i32;

        loop {
            if output_tokens.len() as u32 >= max_tokens {
                break;
            }

            let next_token = sampler.sample(&ctx, -1);
            sampler.accept(next_token);

            if next_token == self.model.token_eos() {
                break;
            }

            output_tokens.push(next_token);

            let mut next_batch = LlamaBatch::new(1, 1);
            next_batch
                .add(next_token, n_cur, &[0], true)
                .map_err(|e| LlmError::Infer(e.to_string()))?;

            ctx.decode(&mut next_batch)
                .map_err(|e| LlmError::Infer(e.to_string()))?;

            n_cur += 1;
        }

        let mut output_bytes = Vec::new();
        for token in output_tokens {
            output_bytes.extend(self.token_to_bytes(token)?);
        }

        let output = String::from_utf8_lossy(&output_bytes).to_string();

        Ok(output)
    }

    pub fn mount_persona_adapter(&self, persona_id: &str) -> Result<bool, LlmError> {
        let adapter_path = self.adapters_dir.join(format!("{persona_id}.gguf"));
        if !adapter_path.exists() {
            return Ok(false);
        }

        let mut adapters = self
            .lora_adapters
            .lock()
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;
        if adapters.contains_key(persona_id) {
            return Ok(true);
        }

        let adapter = self
            .model
            .lora_adapter_init(&adapter_path)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;
        adapters.insert(persona_id.to_string(), adapter);
        Ok(true)
    }

    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>, LlmError> {
        let mut tokens = self
            .model
            .str_to_token(text, llama_cpp_2::model::AddBos::Never)
            .map_err(|e| LlmError::Tokenize(e.to_string()))?;

        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        let max_embedding_tokens = CONTEXT_SIZE as usize;
        if tokens.len() > max_embedding_tokens {
            tokens.truncate(max_embedding_tokens);
        }

        let physical_cores = num_cpus::get_physical() as i32;
        let n_threads = (physical_cores - 1).max(1);
        let context_size = u32::try_from(tokens.len().max(1))
            .unwrap_or(CONTEXT_SIZE)
            .min(CONTEXT_SIZE);

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(context_size))
            .with_n_threads(n_threads)
            .with_n_threads_batch(n_threads)
            .with_embeddings(true)
            .with_pooling_type(LlamaPoolingType::Last);

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;

        let mut batch = LlamaBatch::new(tokens.len(), 1);
        for (i, &token) in tokens.iter().enumerate() {
            batch
                .add(token, i as i32, &[0], i == tokens.len() - 1)
                .map_err(|e| LlmError::Infer(e.to_string()))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| LlmError::Infer(e.to_string()))?;

        let embedding = ctx
            .embeddings_seq_ith(0)
            .or_else(|_| ctx.embeddings_ith((tokens.len() - 1) as i32))
            .map_err(|e| LlmError::Infer(e.to_string()))?;

        let mut vector = embedding.to_vec();
        let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
        if norm > f32::EPSILON {
            for value in &mut vector {
                *value /= norm;
            }
        }

        Ok(vector)
    }
}
