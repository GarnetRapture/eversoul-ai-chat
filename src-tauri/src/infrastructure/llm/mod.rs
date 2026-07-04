use std::path::{Path, PathBuf};

use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, LlamaModel},
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

    pub fn load(app_root: &Path) -> Result<Self, LlmError> {
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
        })
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn infer(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String, LlmError> {
        let max_tokens = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let ctx_params =
            LlamaContextParams::default().with_n_ctx(std::num::NonZeroU32::new(CONTEXT_SIZE));

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;

        let tokens = self
            .model
            .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
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

        let mut sampler = LlamaSampler::greedy();
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
}
