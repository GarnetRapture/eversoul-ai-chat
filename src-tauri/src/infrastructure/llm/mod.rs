pub mod download;
pub mod scheduler;
pub mod streaming;
pub mod validation;
pub mod worker;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use llama_cpp_2::{
    context::params::{LlamaContextParams, LlamaPoolingType},
    context::LlamaContext,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, LlamaLoraAdapter, LlamaModel},
    sampling::LlamaSampler,
    token::LlamaToken,
    TokenToStringError,
};
use thiserror::Error;

use crate::infrastructure::hardware::InferenceProfile;
use crate::startup_debug_log;

pub const QWEN_MODEL_PATH: &str = "ai/model/qwen25-3b-korean-Q4_K_M.gguf";
pub const GEMMA_MODEL_PATH: &str = "ai/model/gemma-2-2b-it-Q4_K_M.gguf";

pub fn get_model_relative_path(active_model: &str) -> &'static str {
    match active_model {
        "gemma" => GEMMA_MODEL_PATH,
        _ => QWEN_MODEL_PATH,
    }
}

pub const EPHEMERAL_CONTEXT_SIZE: u32 = 4096;

#[derive(Debug, Clone)]
pub struct GenerationPlan {
    pub prompt_tokens: Vec<LlamaToken>,
    pub generation_limit: usize,
    pub context_capacity: usize,
    pub truncated_prompt_tokens: usize,
    pub overflowed: bool,
}

#[derive(Debug, Clone)]
pub struct CacheGenerationResult {
    pub text: String,
    pub cached_tokens: Vec<LlamaToken>,
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub reused_prefix_tokens: usize,
    pub truncated_prompt_tokens: usize,
    pub cache_reset: bool,
}

pub struct GenerationRuntime<'a> {
    pub cancel_flag: Option<&'a AtomicBool>,
    pub token_callback: Option<&'a mut dyn FnMut(String) -> Result<(), LlmError>>,
    pub progress_callback: Option<&'a mut dyn FnMut(usize, usize)>,
}

impl<'a> GenerationRuntime<'a> {
    pub fn empty() -> Self {
        Self {
            cancel_flag: None,
            token_callback: None,
            progress_callback: None,
        }
    }

    fn ensure_not_cancelled(&self) -> Result<(), LlmError> {
        if self
            .cancel_flag
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
        {
            return Err(LlmError::Infer("추론 요청이 취소되었습니다.".to_string()));
        }
        Ok(())
    }

    fn emit_token(&mut self, token_text: String) -> Result<(), LlmError> {
        if let Some(callback) = self.token_callback.as_deref_mut() {
            callback(token_text)?;
        }
        Ok(())
    }
}

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
    profile: InferenceProfile,
}

impl LlmEngine {
    fn bounded_generation_limit(&self, requested_max_tokens: u32) -> Result<usize, LlmError> {
        let context_capacity = self.profile.context_size as usize;
        if context_capacity < 2 {
            return Err(LlmError::Infer(format!(
                "LLM 컨텍스트 크기가 너무 작습니다: {}",
                self.profile.context_size
            )));
        }

        let max_generation_tokens = context_capacity - 1;
        Ok((requested_max_tokens as usize)
            .max(1)
            .min(max_generation_tokens))
    }

    fn tokenize_prompt(&self, prompt: &str) -> Result<Vec<LlamaToken>, LlmError> {
        let tokens = self
            .model
            .str_to_token(prompt, llama_cpp_2::model::AddBos::Never)
            .map_err(|e| LlmError::Tokenize(e.to_string()))?;

        if tokens.is_empty() {
            return Err(LlmError::Tokenize(
                "LLM 프롬프트가 토큰을 생성하지 않았습니다.".to_string(),
            ));
        }

        Ok(tokens)
    }

    pub fn generation_plan(
        &self,
        prompt: &str,
        requested_max_tokens: u32,
    ) -> Result<GenerationPlan, LlmError> {
        let mut prompt_tokens = self.tokenize_prompt(prompt)?;
        let original_prompt_len = prompt_tokens.len();
        let generation_limit = self.bounded_generation_limit(requested_max_tokens)?;
        let context_capacity = self.profile.context_size as usize;
        let max_prompt_len = context_capacity - generation_limit;
        let mut truncated_prompt_tokens = 0;

        if prompt_tokens.len() > max_prompt_len {
            let overflow = prompt_tokens.len() - max_prompt_len;
            prompt_tokens.drain(0..overflow);
            truncated_prompt_tokens = overflow;
        }

        Ok(GenerationPlan {
            prompt_tokens,
            generation_limit,
            context_capacity,
            truncated_prompt_tokens,
            overflowed: original_prompt_len > max_prompt_len,
        })
    }

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

    pub fn load(
        app_root: &Path,
        adapters_dir: PathBuf,
        profile: InferenceProfile,
        model_relative_path: &str,
    ) -> Result<Self, LlmError> {
        let model_path = app_root.join(model_relative_path);

        if !model_path.exists() {
            return Err(LlmError::ModelFileNotFound {
                path: model_path.clone(),
            });
        }

        startup_debug_log(&format!("llm_engine:load:start:{}", model_path.display()));
        let backend = LlamaBackend::init().map_err(|e| LlmError::BackendInit(e.to_string()))?;
        startup_debug_log("llm_engine:load:backend_ready");

        let mut model_params = std::pin::pin!(LlamaModelParams::default()
            .with_n_gpu_layers(0)
            .with_use_mmap(false)
            .with_use_mlock(false));
        model_params.as_mut().add_cpu_buft_override(c".*");
        startup_debug_log("llm_engine:load:model_params:cpu_direct_no_mmap");

        let model =
            LlamaModel::load_from_file(&backend, &model_path, model_params.as_ref().get_ref())
                .map_err(|e| LlmError::ModelLoad(e.to_string()))?;
        startup_debug_log("llm_engine:load:model_ready");

        Ok(Self {
            backend,
            model,
            model_path,
            adapters_dir,
            profile,
        })
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn profile(&self) -> InferenceProfile {
        self.profile
    }

    pub fn infer(
        &self,
        prompt: &str,
        max_tokens: Option<u32>,
        persona_id: Option<&str>,
    ) -> Result<String, LlmError> {
        let max_tokens = max_tokens.unwrap_or(self.profile.max_tokens);
        let mut ctx = self.create_context()?;

        let _adapter = if let Some(id) = persona_id {
            self.mount_lora_adapter(&mut ctx, id)?
        } else {
            None
        };

        self.generate_on_context(&mut ctx, prompt, max_tokens)
    }

    pub fn create_context(&self) -> Result<LlamaContext<'_>, LlmError> {
        self.create_context_with_size(self.profile.context_size)
    }

    pub fn create_context_with_size(
        &self,
        context_size: u32,
    ) -> Result<LlamaContext<'_>, LlmError> {
        let n_threads = self.profile.thread_count;

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(context_size))
            .with_n_threads(n_threads)
            .with_n_threads_batch(n_threads);

        self.model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))
    }

    pub fn mount_lora_adapter(
        &self,
        ctx: &mut LlamaContext<'_>,
        persona_id: &str,
    ) -> Result<Option<LlamaLoraAdapter>, LlmError> {
        let adapter_path = self.adapters_dir.join(format!("{persona_id}.gguf"));
        if !adapter_path.exists() {
            return Ok(None);
        }

        let mut adapter = self
            .model
            .lora_adapter_init(&adapter_path)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;
        ctx.lora_adapter_set(&mut adapter, 1.0)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;

        Ok(Some(adapter))
    }

    pub fn generate_on_context(
        &self,
        ctx: &mut LlamaContext<'_>,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<String, LlmError> {
        let mut runtime = GenerationRuntime::empty();
        self.generate_on_context_with_runtime(ctx, prompt, max_tokens, &mut runtime)
    }

    pub fn generate_on_context_with_runtime(
        &self,
        ctx: &mut LlamaContext<'_>,
        prompt: &str,
        max_tokens: u32,
        runtime: &mut GenerationRuntime<'_>,
    ) -> Result<String, LlmError> {
        ctx.clear_kv_cache();

        let plan = self.generation_plan(prompt, max_tokens)?;
        let tokens = plan.prompt_tokens;

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
            runtime.ensure_not_cancelled()?;
            if output_tokens.len() >= plan.generation_limit
                || n_cur as usize >= plan.context_capacity
            {
                break;
            }

            let next_token = sampler.sample(ctx, -1);
            sampler.accept(next_token);

            if next_token == self.model.token_eos() {
                break;
            }

            output_tokens.push(next_token);
            runtime.emit_token(
                String::from_utf8_lossy(&self.token_to_bytes(next_token)?).to_string(),
            )?;

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

        Ok(String::from_utf8_lossy(&output_bytes).to_string())
    }

    pub fn count_tokens(&self, text: &str) -> Result<usize, LlmError> {
        let tokens = self
            .model
            .str_to_token(text, llama_cpp_2::model::AddBos::Never)
            .map_err(|e| LlmError::Tokenize(e.to_string()))?;
        Ok(tokens.len())
    }

    pub fn generate_with_cache(
        &self,
        ctx: &mut LlamaContext<'_>,
        cached_tokens: &[LlamaToken],
        prompt: &str,
        max_tokens: u32,
    ) -> Result<CacheGenerationResult, LlmError> {
        let mut runtime = GenerationRuntime::empty();
        self.generate_with_cache_runtime(ctx, cached_tokens, prompt, max_tokens, &mut runtime)
    }

    pub fn generate_with_cache_runtime(
        &self,
        ctx: &mut LlamaContext<'_>,
        cached_tokens: &[LlamaToken],
        prompt: &str,
        max_tokens: u32,
        runtime: &mut GenerationRuntime<'_>,
    ) -> Result<CacheGenerationResult, LlmError> {
        let plan = self.generation_plan(prompt, max_tokens)?;
        let prompt_tokens = plan.prompt_tokens;
        let truncated_prompt_tokens = plan.truncated_prompt_tokens;
        let mut cache_reset = false;

        if plan.overflowed {
            ctx.clear_kv_cache();
            cache_reset = true;
        }

        let effective_cached_tokens: &[LlamaToken] =
            if plan.overflowed { &[] } else { cached_tokens };

        let matching_len = effective_cached_tokens
            .iter()
            .zip(prompt_tokens.iter())
            .take_while(|(cached, incoming)| cached == incoming)
            .count();
        let mut common_len = matching_len.min(prompt_tokens.len().saturating_sub(1));

        if common_len < effective_cached_tokens.len() {
            let removed = ctx
                .clear_kv_cache_seq(Some(0), Some(common_len as u32), None)
                .map_err(|e| LlmError::ContextCreate(e.to_string()))?;
            if !removed {
                ctx.clear_kv_cache();
                cache_reset = true;
                common_len = 0;
            }
        }

        let new_tokens = &prompt_tokens[common_len..];
        if !new_tokens.is_empty() {
            let mut batch = LlamaBatch::new(new_tokens.len(), 1);
            for (offset, &token) in new_tokens.iter().enumerate() {
                let position = (common_len + offset) as i32;
                let is_last = offset == new_tokens.len() - 1;
                batch
                    .add(token, position, &[0], is_last)
                    .map_err(|e| LlmError::Infer(e.to_string()))?;
            }

            ctx.decode(&mut batch)
                .map_err(|e| LlmError::Infer(e.to_string()))?;
        }

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
        let mut output_tokens: Vec<LlamaToken> = Vec::new();
        let mut n_cur = prompt_tokens.len() as i32;

        loop {
            runtime.ensure_not_cancelled()?;
            if output_tokens.len() >= plan.generation_limit
                || n_cur as usize >= plan.context_capacity
            {
                break;
            }

            let next_token = sampler.sample(ctx, -1);
            sampler.accept(next_token);

            if next_token == self.model.token_eos() {
                break;
            }

            output_tokens.push(next_token);
            runtime.emit_token(
                String::from_utf8_lossy(&self.token_to_bytes(next_token)?).to_string(),
            )?;

            let mut next_batch = LlamaBatch::new(1, 1);
            next_batch
                .add(next_token, n_cur, &[0], true)
                .map_err(|e| LlmError::Infer(e.to_string()))?;

            ctx.decode(&mut next_batch)
                .map_err(|e| LlmError::Infer(e.to_string()))?;

            n_cur += 1;
        }

        let mut output_bytes = Vec::new();
        for &token in &output_tokens {
            output_bytes.extend(self.token_to_bytes(token)?);
        }

        let prompt_token_count = prompt_tokens.len();
        let generated_token_count = output_tokens.len();
        let mut full_sequence = prompt_tokens;
        full_sequence.extend(output_tokens);

        Ok(CacheGenerationResult {
            text: String::from_utf8_lossy(&output_bytes).to_string(),
            prompt_tokens: prompt_token_count,
            generated_tokens: generated_token_count,
            cached_tokens: full_sequence,
            reused_prefix_tokens: common_len,
            truncated_prompt_tokens,
            cache_reset,
        })
    }

    pub fn prefill_cache_runtime(
        &self,
        ctx: &mut LlamaContext<'_>,
        cached_tokens: &[LlamaToken],
        prompt: &str,
        runtime: &mut GenerationRuntime<'_>,
    ) -> Result<CacheGenerationResult, LlmError> {
        let plan = self.generation_plan(prompt, 1)?;
        let prompt_tokens = plan.prompt_tokens;
        let truncated_prompt_tokens = plan.truncated_prompt_tokens;
        let mut cache_reset = false;

        if plan.overflowed {
            ctx.clear_kv_cache();
            cache_reset = true;
        }

        let effective_cached_tokens: &[LlamaToken] =
            if plan.overflowed { &[] } else { cached_tokens };
        let matching_len = effective_cached_tokens
            .iter()
            .zip(prompt_tokens.iter())
            .take_while(|(cached, incoming)| cached == incoming)
            .count();
        let mut common_len = matching_len.min(prompt_tokens.len());

        if common_len < effective_cached_tokens.len() {
            let removed = ctx
                .clear_kv_cache_seq(Some(0), Some(common_len as u32), None)
                .map_err(|e| LlmError::ContextCreate(e.to_string()))?;
            if !removed {
                ctx.clear_kv_cache();
                cache_reset = true;
                common_len = 0;
            }
        }

        let new_tokens = &prompt_tokens[common_len..];
        if !new_tokens.is_empty() {
            let n_batch = ctx.n_batch() as usize;
            let mut processed = 0;
            let total = new_tokens.len();

            for chunk in new_tokens.chunks(n_batch) {
                runtime.ensure_not_cancelled()?;
                let mut batch = LlamaBatch::new(chunk.len(), 1);
                
                for (offset, &token) in chunk.iter().enumerate() {
                    let position = (common_len + processed + offset) as i32;
                    let is_last = (processed + offset) == total - 1;
                    batch
                        .add(token, position, &[0], is_last)
                        .map_err(|e| LlmError::Infer(e.to_string()))?;
                }

                ctx.decode(&mut batch)
                    .map_err(|e| LlmError::Infer(e.to_string()))?;
                
                processed += chunk.len();
                if let Some(cb) = runtime.progress_callback.as_deref_mut() {
                    cb(processed, total);
                }
            }
        } else {
            if let Some(cb) = runtime.progress_callback.as_deref_mut() {
                cb(prompt_tokens.len(), prompt_tokens.len());
            }
        }

        let prompt_token_count = prompt_tokens.len();
        Ok(CacheGenerationResult {
            text: String::new(),
            cached_tokens: prompt_tokens,
            prompt_tokens: prompt_token_count,
            generated_tokens: 0,
            reused_prefix_tokens: common_len,
            truncated_prompt_tokens,
            cache_reset,
        })
    }

    pub fn mount_persona_adapter(&self, persona_id: &str) -> Result<bool, LlmError> {
        let adapter_path = self.adapters_dir.join(format!("{persona_id}.gguf"));
        if !adapter_path.exists() {
            return Ok(false);
        }

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

        let max_embedding_tokens = self.profile.context_size as usize;
        if tokens.len() > max_embedding_tokens {
            tokens.truncate(max_embedding_tokens);
        }

        let n_threads = self.profile.thread_count;
        let context_size = u32::try_from(tokens.len().max(1))
            .unwrap_or(self.profile.context_size)
            .min(self.profile.context_size);

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::infrastructure::hardware::InferenceProfile;
    use crate::infrastructure::llm::validation::validate_model_file;
    use crate::infrastructure::llm::worker::LlmWorkerHandle;

    use super::{LlmEngine, QWEN_MODEL_PATH};

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    fn test_profile() -> InferenceProfile {
        InferenceProfile {
            thread_count: 1,
            context_size: 1024,
            max_tokens: 8,
            max_active_sessions: 2,
        }
    }

    #[test]
    fn llm_model_file_validation_uses_real_model() {
        let model_path = project_root().join(QWEN_MODEL_PATH);
        let validation = validate_model_file(&model_path).expect("실제 GGUF 모델 파일 검증 실패");
        assert!(validation.size_bytes > 0);
        assert_eq!(validation.sha256.len(), 64);
        if let Some(hash_matches_sidecar) = validation.hash_matches_sidecar {
            assert!(hash_matches_sidecar);
        }
    }

    #[test]
    fn llm_real_model_generates_text() {
        let root = project_root();
        let engine = LlmEngine::load(&root, root.join("lora_adapters"), test_profile(), QWEN_MODEL_PATH)
            .expect("실제 GGUF 모델 로드 실패");
        let output = engine
            .infer(
                "<|im_start|>system\n한국어로 한 단어만 답하십시오.<|im_end|>\n<|im_start|>user\n인사<|im_end|>\n<|im_start|>assistant\n",
                Some(4),
                None,
            )
            .expect("실제 GGUF 모델 추론 실패");
        assert!(!output.trim().is_empty());
    }

    #[test]
    fn llm_worker_loads_real_model_on_dedicated_stack() {
        let root = project_root();
        let handle = LlmWorkerHandle::load_and_spawn(
            root.clone(),
            root.join("lora_adapters"),
            test_profile(),
            QWEN_MODEL_PATH,
        )
        .expect("전용 워커 스택에서 실제 GGUF 모델 로드 실패");
        let output = handle
            .infer(
                "<|im_start|>system\n한국어로 한 단어만 답하십시오.<|im_end|>\n<|im_start|>user\n상태<|im_end|>\n<|im_start|>assistant\n",
                Some(4),
                None,
            )
            .expect("전용 워커 스택 실제 모델 추론 실패");
        assert!(!output.trim().is_empty());
    }

    #[test]
    fn llm_kv_cache_reuse_regression_uses_real_context() {
        let root = project_root();
        let engine = LlmEngine::load(&root, root.join("lora_adapters"), test_profile(), QWEN_MODEL_PATH)
            .expect("실제 GGUF 모델 로드 실패");
        let mut ctx = engine.create_context().expect("LLM 컨텍스트 생성 실패");
        let prompt = "<|im_start|>system\n한국어로 짧게 답하십시오.<|im_end|>\n<|im_start|>user\n테스트<|im_end|>\n<|im_start|>assistant\n";
        let first = engine
            .generate_with_cache(&mut ctx, &[], prompt, 1)
            .expect("첫 번째 KV 캐시 추론 실패");
        let second = engine
            .generate_with_cache(&mut ctx, &first.cached_tokens, prompt, 1)
            .expect("두 번째 KV 캐시 추론 실패");
        assert!(second.reused_prefix_tokens > 0);
    }
}
