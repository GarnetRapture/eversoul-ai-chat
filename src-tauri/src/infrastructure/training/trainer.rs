use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use candle_core::quantized::gguf_file;
use candle_core::{DType, Device, Tensor, D};
use candle_nn::optim::{AdamW, Optimizer, ParamsAdamW};
use candle_nn::VarBuilder;
use tokenizers::models::unigram::Unigram;
use tokenizers::pre_tokenizers::metaspace::Metaspace;
use tokenizers::processors::template::TemplateProcessing;
use tokenizers::Tokenizer;

use super::config::Gemma2Config;
use super::dataset::{tokenize_example, ConversationExample};
use super::gguf_export::export_lora_to_gguf;
use super::lora::{new_lora_varmap, save_lora_weights};
use super::model::Gemma2Model;

const LORA_RANK: usize = 8;
const LORA_ALPHA: f64 = 16.0;
const LEARNING_RATE: f64 = 1e-4;
const SUPPORTED_GGUF_ARCHITECTURE: &str = "gemma2";

pub struct TrainingReport {
    pub steps: usize,
    pub final_loss: f32,
    pub adapter_path: std::path::PathBuf,
    pub gguf_adapter_path: std::path::PathBuf,
}

/// LoRA 학습 원본 텐서 명명(HuggingFace 방식: `model.layers.N.self_attn.q_proj.weight`)과
/// GGUF/llama.cpp 텐서 명명(`blk.N.attn_q.weight`)이 서로 다르므로, GGUF에서 역양자화한
/// 텐서를 HF 키로 재매핑해 Gemma2Model::load(VarBuilder)에 그대로 넣는다. 텐서 이름은
/// llama.cpp 런타임 소스(`src/models/gemma2.cpp`, `src/llama-arch.cpp`) 기준.
/// Config 자체도 이 GGUF의 실제 메타데이터에서 읽어(`Gemma2Config::from_gguf_metadata`)
/// 하드코딩 추측이 실제 배포 파일과 어긋나는 사고를 막는다.
fn load_base_tensors_from_gguf(
    gguf_path: &std::path::Path,
    device: &Device,
) -> anyhow::Result<(Gemma2Config, HashMap<String, Tensor>)> {
    let mut reader = BufReader::new(File::open(gguf_path).map_err(|e| {
        anyhow::anyhow!("베이스 GGUF 모델을 열 수 없습니다({}): {e}", gguf_path.display())
    })?);
    let content = gguf_file::Content::read(&mut reader)
        .map_err(|e| anyhow::anyhow!("GGUF 파싱 실패: {e}"))?;

    let architecture = content
        .metadata
        .get("general.architecture")
        .and_then(|v| v.to_string().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();
    if architecture != SUPPORTED_GGUF_ARCHITECTURE {
        anyhow::bail!(
            "architecture_mismatch:{}:{}",
            SUPPORTED_GGUF_ARCHITECTURE,
            architecture
        );
    }

    let cfg = Gemma2Config::from_gguf_metadata(&content.metadata)?;

    let mut fetch = |gguf_name: &str| -> anyhow::Result<Tensor> {
        let qtensor = content
            .tensor(&mut reader, gguf_name, device)
            .map_err(|e| anyhow::anyhow!("GGUF 텐서 로드 실패({gguf_name}): {e}"))?;
        Ok(qtensor.dequantize(device)?.to_dtype(DType::F32)?)
    };

    let mut tensors = HashMap::new();
    tensors.insert(
        "model.embed_tokens.weight".to_string(),
        fetch("token_embd.weight")?,
    );
    for i in 0..cfg.num_hidden_layers {
        tensors.insert(
            format!("model.layers.{i}.self_attn.q_proj.weight"),
            fetch(&format!("blk.{i}.attn_q.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.self_attn.k_proj.weight"),
            fetch(&format!("blk.{i}.attn_k.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.self_attn.v_proj.weight"),
            fetch(&format!("blk.{i}.attn_v.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.self_attn.o_proj.weight"),
            fetch(&format!("blk.{i}.attn_output.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.mlp.gate_proj.weight"),
            fetch(&format!("blk.{i}.ffn_gate.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.mlp.up_proj.weight"),
            fetch(&format!("blk.{i}.ffn_up.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.mlp.down_proj.weight"),
            fetch(&format!("blk.{i}.ffn_down.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.input_layernorm.weight"),
            fetch(&format!("blk.{i}.attn_norm.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.post_attention_layernorm.weight"),
            fetch(&format!("blk.{i}.attn_post_norm.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.pre_feedforward_layernorm.weight"),
            fetch(&format!("blk.{i}.ffn_norm.weight"))?,
        );
        tensors.insert(
            format!("model.layers.{i}.post_feedforward_layernorm.weight"),
            fetch(&format!("blk.{i}.ffn_post_norm.weight"))?,
        );
    }
    tensors.insert(
        "model.norm.weight".to_string(),
        fetch("output_norm.weight")?,
    );
    if let Ok(lm_head) = fetch("output.weight") {
        tensors.insert("lm_head.weight".to_string(), lm_head);
    }

    Ok((cfg, tensors))
}

/// Gemma 2는 SentencePiece 유니그램 토크나이저를 쓰며, GGUF는 그 vocab/점수를
/// `tokenizer.ggml.tokens`(문자열 배열)·`tokenizer.ggml.scores`(로그확률 배열)로
/// 내장하고 있다(llama.cpp GGUF 표준). 별도의 HuggingFace tokenizer.json 없이
/// 이 GGUF 하나만으로 정확히 동일한 vocab의 토크나이저를 재구성한다.
fn build_tokenizer_from_gguf(content: &gguf_file::Content) -> anyhow::Result<Tokenizer> {
    let tokens = content
        .metadata
        .get("tokenizer.ggml.tokens")
        .ok_or_else(|| anyhow::anyhow!("GGUF에 tokenizer.ggml.tokens가 없습니다"))?
        .to_vec()
        .map_err(|e| anyhow::anyhow!("tokenizer.ggml.tokens 파싱 실패: {e}"))?;
    let scores = content
        .metadata
        .get("tokenizer.ggml.scores")
        .and_then(|v| v.to_vec().ok());

    let mut vocab: Vec<(String, f64)> = Vec::with_capacity(tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        let piece = token
            .to_string()
            .map_err(|e| anyhow::anyhow!("tokenizer.ggml.tokens[{i}] 파싱 실패: {e}"))?
            .clone();
        let score = scores
            .as_ref()
            .and_then(|s| s.get(i))
            .and_then(|v| v.to_f32().ok())
            .unwrap_or(0.0) as f64;
        vocab.push((piece, score));
    }

    let unk_id = content
        .metadata
        .get("tokenizer.ggml.unknown_token_id")
        .and_then(|v| v.to_u32().ok())
        .map(|v| v as usize);

    // 학습용 프롬프트(`<start_of_turn>user\n...`)와 채팅 추론 쪽 실제 생성 토큰화
    // (infrastructure/llm/mod.rs의 AddBos::Always)가 동일하게 BOS로 시작하도록,
    // 인코딩 시 BOS를 자동으로 앞에 붙이는 post-processor를 등록한다.
    let bos_id = content
        .metadata
        .get("tokenizer.ggml.bos_token_id")
        .and_then(|v| v.to_u32().ok());

    let unigram = Unigram::from(vocab.clone(), unk_id, false)
        .map_err(|e| anyhow::anyhow!("유니그램 토크나이저 구성 실패: {e}"))?;

    let mut tokenizer = Tokenizer::new(unigram);
    tokenizer.with_pre_tokenizer(Some(Metaspace::default()));

    if let Some(bos_id) = bos_id {
        let bos_piece = vocab
            .get(bos_id as usize)
            .map(|(piece, _)| piece.clone())
            .ok_or_else(|| anyhow::anyhow!("bos_token_id({bos_id})가 vocab 범위를 벗어났습니다"))?;
        let post_processor = TemplateProcessing::builder()
            .try_single(format!("{bos_piece} $A"))
            .map_err(|e| anyhow::anyhow!("BOS 템플릿 구성 실패: {e}"))?
            .special_tokens(vec![(bos_piece.as_str(), bos_id)])
            .build()
            .map_err(|e| anyhow::anyhow!("BOS post-processor 빌드 실패: {e}"))?;
        tokenizer.with_post_processor(Some(post_processor));
    }

    Ok(tokenizer)
}

pub fn train_persona_lora(
    examples: Vec<ConversationExample>,
    base_model_gguf_path: &std::path::Path,
    output_path: &std::path::Path,
    mut progress_callback: impl FnMut(usize, usize, f32),
) -> anyhow::Result<TrainingReport> {
    if examples.is_empty() {
        anyhow::bail!("학습할 대화 예시가 없습니다. 먼저 대화를 충분히 누적하십시오.");
    }

    let device = Device::Cpu;

    let mut gguf_reader = BufReader::new(File::open(base_model_gguf_path).map_err(|e| {
        anyhow::anyhow!(
            "베이스 GGUF 모델을 열 수 없습니다({}): {e}",
            base_model_gguf_path.display()
        )
    })?);
    let gguf_content = gguf_file::Content::read(&mut gguf_reader)
        .map_err(|e| anyhow::anyhow!("GGUF 파싱 실패: {e}"))?;
    let tokenizer = build_tokenizer_from_gguf(&gguf_content)?;

    let (cfg, base_tensors) = load_base_tensors_from_gguf(base_model_gguf_path, &device)?;
    let vb = VarBuilder::from_tensors(base_tensors, DType::F32, &device);

    let lora_varmap = new_lora_varmap();
    let model = Gemma2Model::load(&cfg, vb, &lora_varmap, LORA_RANK, LORA_ALPHA, &device)?;

    let params = ParamsAdamW {
        lr: LEARNING_RATE,
        ..Default::default()
    };
    let mut optimizer = AdamW::new(lora_varmap.all_vars(), params)?;

    let mut final_loss = 0f32;
    let mut steps = 0usize;
    let total_steps = examples.len();

    for example in &examples {
        let batch = tokenize_example(&tokenizer, example, &device)?;

        let logits = model.forward(&batch.input_ids)?;
        let (_b, seq_len, vocab_size) = logits.dims3()?;
        let logits = logits.reshape((seq_len, vocab_size))?;

        let log_probs = candle_nn::ops::log_softmax(&logits, D::Minus1)?;
        let picked = log_probs
            .gather(&batch.labels.unsqueeze(1)?, 1)?
            .squeeze(1)?;
        let loss_per_token = picked.neg()?;
        let masked = (loss_per_token * &batch.loss_mask)?;
        let valid_count = batch.loss_mask.sum_all()?;
        let loss = (masked.sum_all()? / valid_count)?;

        optimizer.backward_step(&loss)?;

        final_loss = loss.to_scalar::<f32>()?;
        steps += 1;
        progress_callback(steps, total_steps, final_loss);
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    save_lora_weights(&lora_varmap, output_path)?;

    let gguf_adapter_path = output_path.with_extension("gguf");
    export_lora_to_gguf(&lora_varmap, &cfg, LORA_ALPHA, &gguf_adapter_path)?;

    Ok(TrainingReport {
        steps,
        final_loss,
        adapter_path: output_path.to_path_buf(),
        gguf_adapter_path,
    })
}
