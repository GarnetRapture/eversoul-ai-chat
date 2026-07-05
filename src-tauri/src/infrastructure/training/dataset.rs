use candle_core::{Device, Tensor};
use tokenizers::Tokenizer;

pub struct ConversationExample {
    pub system_prompt: String,
    pub prompt_turns: Vec<(String, String)>,
    pub target_reply: String,
}

pub struct TokenizedBatch {
    pub input_ids: Tensor,
    pub labels: Tensor,
    pub loss_mask: Tensor,
}

const IGNORE_INDEX: i64 = -100;

pub fn tokenize_example(
    tokenizer: &Tokenizer,
    example: &ConversationExample,
    device: &Device,
) -> anyhow::Result<TokenizedBatch> {
    let mut prompt = format!("<|im_start|>system\n{}<|im_end|>\n", example.system_prompt);
    for (role, content) in &example.prompt_turns {
        prompt.push_str(&format!("<|im_start|>{role}\n{content}<|im_end|>\n"));
    }
    prompt.push_str("<|im_start|>assistant\n");

    let prompt_ids = tokenizer
        .encode(prompt, true)
        .map_err(|e| anyhow::anyhow!("프롬프트 토큰화 실패: {e}"))?
        .get_ids()
        .to_vec();

    let target = format!("{}<|im_end|>\n", example.target_reply);
    let target_ids = tokenizer
        .encode(target, false)
        .map_err(|e| anyhow::anyhow!("정답 응답 토큰화 실패: {e}"))?
        .get_ids()
        .to_vec();

    let mut input_ids: Vec<u32> = Vec::with_capacity(prompt_ids.len() + target_ids.len());
    input_ids.extend_from_slice(&prompt_ids);
    input_ids.extend_from_slice(&target_ids);

    let mut labels: Vec<i64> = vec![IGNORE_INDEX; prompt_ids.len().saturating_sub(1)];
    for &id in target_ids.iter() {
        labels.push(id as i64);
    }
    labels.push(IGNORE_INDEX);

    let seq_len = input_ids.len() - 1;
    let input_ids = &input_ids[..seq_len];
    let labels = &labels[..seq_len.min(labels.len())];

    let mask: Vec<f32> = labels
        .iter()
        .map(|&l| if l == IGNORE_INDEX { 0.0 } else { 1.0 })
        .collect();
    let labels_u32: Vec<u32> = labels
        .iter()
        .map(|&l| if l == IGNORE_INDEX { 0 } else { l as u32 })
        .collect();

    let input_ids = Tensor::from_vec(input_ids.to_vec(), (1, seq_len), device)?;
    let labels = Tensor::from_vec(labels_u32, (seq_len,), device)?;
    let loss_mask = Tensor::from_vec(mask, (seq_len,), device)?;

    Ok(TokenizedBatch {
        input_ids,
        labels,
        loss_mask,
    })
}
