use std::path::{Path, PathBuf};

use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, LlamaModel},
    token::data_array::LlamaTokenDataArray,
};
use thiserror::Error;

/// 프로젝트 루트 기준 로컬 LLM 모델 파일 경로.
/// 실제 파일은 git 제외 대상이며 로컬에 별도 배치해야 한다.
pub const MODEL_RELATIVE_PATH: &str = "ai/model/qwen25-3b-korean-Q4_K_M.gguf";

/// LLM 추론 최대 컨텍스트 토큰 수.
const CONTEXT_SIZE: u32 = 2048;

/// LLM 추론 기본 최대 생성 토큰 수.
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

/// CPU 기반 로컬 LLM 엔진.
///
/// Qwen2.5-3B-Korean Q4_K_M GGUF 모델을 llama.cpp를 통해 로딩하고
/// CPU에서 텍스트 생성 추론을 수행한다.
pub struct LlmEngine {
    backend: LlamaBackend,
    model: LlamaModel,
    model_path: PathBuf,
}

impl LlmEngine {
    /// 주어진 앱 루트 디렉터리 기준으로 GGUF 모델을 로딩해 `LlmEngine`을 생성한다.
    ///
    /// # Arguments
    /// * `app_root` - 앱 실행 파일이 위치한 디렉터리 (또는 개발 중 프로젝트 루트)
    pub fn load(app_root: &Path) -> Result<Self, LlmError> {
        let model_path = app_root.join(MODEL_RELATIVE_PATH);

        if !model_path.exists() {
            return Err(LlmError::ModelFileNotFound {
                path: model_path.clone(),
            });
        }

        let backend = LlamaBackend::init()
            .map_err(|e| LlmError::BackendInit(e.to_string()))?;

        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;

        Ok(Self {
            backend,
            model,
            model_path,
        })
    }

    /// 현재 로딩된 모델 파일의 절대 경로를 반환한다.
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    /// 주어진 프롬프트로 텍스트 생성 추론을 수행하고 결과 문자열을 반환한다.
    ///
    /// # Arguments
    /// * `prompt` - 입력 프롬프트 문자열
    /// * `max_tokens` - 최대 생성 토큰 수. `None`이면 `DEFAULT_MAX_TOKENS`를 사용한다.
    pub fn infer(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String, LlmError> {
        let max_tokens = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(CONTEXT_SIZE));



        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;

        // 프롬프트 토큰화
        let tokens = self
            .model
            .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
            .map_err(|e| LlmError::Tokenize(e.to_string()))?;

        // 배치 생성 및 프롬프트 토큰 추가
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

        // 토큰 생성 루프
        let mut output_tokens: Vec<llama_cpp_2::token::LlamaToken> = Vec::new();
        let mut n_cur = n_tokens as i32;

        loop {
            if output_tokens.len() as u32 >= max_tokens {
                break;
            }

            let candidates = ctx.candidates();
            let mut candidates_p = LlamaTokenDataArray::from_iter(candidates, false);

            // 탐욕적 샘플링 (greedy)
            let next_token = ctx.sample_token_greedy(&mut candidates_p);

            // EOS(End of Sequence) 토큰이면 종료
            if next_token == self.model.token_eos() {
                break;
            }

            output_tokens.push(next_token);

            // 다음 추론을 위한 단일 토큰 배치
            let mut next_batch = LlamaBatch::new(1, 1);
            next_batch
                .add(next_token, n_cur, &[0], true)
                .map_err(|e| LlmError::Infer(e.to_string()))?;

            ctx.decode(&mut next_batch)
                .map_err(|e| LlmError::Infer(e.to_string()))?;

            n_cur += 1;
        }

        // 생성된 토큰을 문자열로 디코딩
        let output: String = output_tokens
            .iter()
            .map(|&token| {
                self.model
                    .token_to_str(token, llama_cpp_2::model::Special::Tokenize)
                    .unwrap_or_default()
            })
            .collect();

        Ok(output)
    }
}
