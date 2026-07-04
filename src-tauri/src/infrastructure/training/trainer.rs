use candle_core::{DType, Device, D};
use candle_nn::optim::{AdamW, Optimizer, ParamsAdamW};
use candle_nn::VarBuilder;
use tokenizers::Tokenizer;

use super::config::Qwen2Config;
use super::dataset::{tokenize_example, ConversationExample};
use super::downloader::ensure_base_model_files;
use super::lora::{new_lora_varmap, save_lora_weights};
use super::model::Qwen2Model;

const LORA_RANK: usize = 8;
const LORA_ALPHA: f64 = 16.0;
const LEARNING_RATE: f64 = 1e-4;

pub struct TrainingReport {
    pub steps: usize,
    pub final_loss: f32,
    pub adapter_path: std::path::PathBuf,
}

pub fn train_persona_lora(
    examples: Vec<ConversationExample>,
    output_path: &std::path::Path,
) -> anyhow::Result<TrainingReport> {
    if examples.is_empty() {
        anyhow::bail!("학습할 대화 예시가 없습니다. 먼저 대화를 충분히 누적하십시오.");
    }

    let device = Device::Cpu;
    let cfg = Qwen2Config::default();

    let base_files = ensure_base_model_files()?;
    let tokenizer = Tokenizer::from_file(&base_files.tokenizer)
        .map_err(|e| anyhow::anyhow!("토크나이저 로드 실패: {e}"))?;

    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&base_files.safetensors, DType::F32, &device)?
    };

    let lora_varmap = new_lora_varmap();
    let model = Qwen2Model::load(&cfg, vb, &lora_varmap, LORA_RANK, LORA_ALPHA, &device)?;

    let params = ParamsAdamW {
        lr: LEARNING_RATE,
        ..Default::default()
    };
    let mut optimizer = AdamW::new(lora_varmap.all_vars(), params)?;

    let mut final_loss = 0f32;
    let mut steps = 0usize;

    for example in &examples {
        let batch = tokenize_example(&tokenizer, example, &device)?;

        let logits = model.forward(&batch.input_ids)?;
        let (_b, seq_len, vocab_size) = logits.dims3()?;
        let logits = logits.reshape((seq_len, vocab_size))?;

        let log_probs = candle_nn::ops::log_softmax(&logits, D::Minus1)?;
        let picked = log_probs.gather(&batch.labels.unsqueeze(1)?, 1)?.squeeze(1)?;
        let loss_per_token = picked.neg()?;
        let masked = (loss_per_token * &batch.loss_mask)?;
        let valid_count = batch.loss_mask.sum_all()?;
        let loss = (masked.sum_all()? / valid_count)?;

        optimizer.backward_step(&loss)?;

        final_loss = loss.to_scalar::<f32>()?;
        steps += 1;
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    save_lora_weights(&lora_varmap, output_path)?;

    Ok(TrainingReport {
        steps,
        final_loss,
        adapter_path: output_path.to_path_buf(),
    })
}
