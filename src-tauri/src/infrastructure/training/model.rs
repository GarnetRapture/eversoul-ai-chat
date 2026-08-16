use candle_core::{DType, Device, Result, Tensor, D};
use candle_nn::{Embedding, Linear, Module, VarBuilder, VarMap};

use super::config::Gemma2Config;
use super::lora::LoraLinear;

struct RmsNorm {
    weight: Tensor,
    eps: f64,
}

impl RmsNorm {
    /// Gemma/Gemma2는 `x_normed * (1 + weight)`를 쓴다(가중치를 0으로 초기화해
    /// 학습 시작 시점을 항등 변환으로 만들기 위함) — 표준 RMSNorm과 다른 부분이다.
    fn load(size: usize, eps: f64, vb: VarBuilder) -> Result<Self> {
        let weight = vb.get(size, "weight")?;
        Ok(Self { weight, eps })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let in_dtype = x.dtype();
        let x = x.to_dtype(DType::F32)?;
        let variance = x.sqr()?.mean_keepdim(D::Minus1)?;
        let x_normed = x.broadcast_div(&(variance + self.eps)?.sqrt()?)?;
        let x_normed = x_normed.to_dtype(in_dtype)?;
        let scale = (self.weight.ones_like()? + &self.weight)?;
        x_normed.broadcast_mul(&scale)
    }
}

struct RotaryEmbedding {
    cos: Tensor,
    sin: Tensor,
}

impl RotaryEmbedding {
    fn new(head_dim: usize, rope_theta: f32, max_position_embeddings: usize, device: &Device) -> Result<Self> {
        let theta: Vec<f32> = (0..head_dim)
            .step_by(2)
            .map(|i| 1f32 / rope_theta.powf(i as f32 / head_dim as f32))
            .collect();
        let theta = Tensor::new(theta.as_slice(), device)?;
        let idx = Tensor::arange(0u32, max_position_embeddings as u32, device)?
            .to_dtype(DType::F32)?
            .reshape((max_position_embeddings, 1))?;
        let freqs = idx.broadcast_matmul(&theta.reshape((1, theta.elem_count()))?)?;
        let freqs = Tensor::cat(&[&freqs, &freqs], D::Minus1)?;
        Ok(Self {
            cos: freqs.cos()?,
            sin: freqs.sin()?,
        })
    }

    fn apply(&self, x: &Tensor, seqlen_offset: usize) -> Result<Tensor> {
        let (_b, _h, seq_len, _d) = x.dims4()?;
        let cos = self.cos.narrow(0, seqlen_offset, seq_len)?;
        let sin = self.sin.narrow(0, seqlen_offset, seq_len)?;
        let rotated = rotate_half(x)?;
        x.broadcast_mul(&cos)? + rotated.broadcast_mul(&sin)?
    }
}

fn rotate_half(x: &Tensor) -> Result<Tensor> {
    let last_dim = x.dim(D::Minus1)?;
    let x1 = x.narrow(D::Minus1, 0, last_dim / 2)?;
    let x2 = x.narrow(D::Minus1, last_dim / 2, last_dim / 2)?;
    Tensor::cat(&[&x2.neg()?, &x1], D::Minus1)
}

fn repeat_kv(x: Tensor, n_rep: usize) -> Result<Tensor> {
    if n_rep == 1 {
        return Ok(x);
    }
    let (b, n_kv_heads, seq_len, head_dim) = x.dims4()?;
    x.unsqueeze(2)?
        .expand((b, n_kv_heads, n_rep, seq_len, head_dim))?
        .reshape((b, n_kv_heads * n_rep, seq_len, head_dim))
}

fn causal_mask(seq_len: usize, device: &Device) -> Result<Tensor> {
    let mask: Vec<f32> = (0..seq_len)
        .flat_map(|i| (0..seq_len).map(move |j| if j > i { f32::NEG_INFINITY } else { 0.0 }))
        .collect();
    Tensor::from_vec(mask, (1, 1, seq_len, seq_len), device)
}

/// `cap * tanh(x / cap)` — Gemma2가 attention logit과 최종 logit 양쪽에 쓰는
/// softcapping. 공식 스펙(HF `Gemma2Config`, candle-transformers `models/gemma2.rs`).
fn softcap(x: &Tensor, cap: f64) -> Result<Tensor> {
    ((x / cap)?.tanh()? * cap)?.to_dtype(x.dtype())
}

struct Gemma2Attention {
    q_proj: LoraLinear,
    k_proj: Linear,
    v_proj: LoraLinear,
    o_proj: Linear,
    num_heads: usize,
    num_kv_heads: usize,
    head_dim: usize,
    attn_scale: f64,
    attn_logit_softcapping: f64,
}

impl Gemma2Attention {
    fn load(
        cfg: &Gemma2Config,
        vb: VarBuilder,
        lora_varmap: &VarMap,
        lora_rank: usize,
        lora_alpha: f64,
        layer_idx: usize,
        device: &Device,
    ) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let head_dim = cfg.head_dim;
        let q_dim = cfg.num_attention_heads * head_dim;
        let kv_dim = cfg.num_key_value_heads * head_dim;

        let q_weight = vb.get((q_dim, hidden), "q_proj.weight")?;
        let q_proj = LoraLinear::new(
            q_weight,
            hidden,
            q_dim,
            lora_rank,
            lora_alpha,
            lora_varmap,
            &format!("layer{layer_idx}.q_proj"),
            device,
        )?;
        let k_proj = Linear::new(vb.get((kv_dim, hidden), "k_proj.weight")?, None);
        let v_weight = vb.get((kv_dim, hidden), "v_proj.weight")?;
        let v_proj = LoraLinear::new(
            v_weight,
            hidden,
            kv_dim,
            lora_rank,
            lora_alpha,
            lora_varmap,
            &format!("layer{layer_idx}.v_proj"),
            device,
        )?;
        let o_proj = Linear::new(vb.get((hidden, q_dim), "o_proj.weight")?, None);

        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            num_heads: cfg.num_attention_heads,
            num_kv_heads: cfg.num_key_value_heads,
            head_dim,
            attn_scale: 1f64 / cfg.query_pre_attn_scalar.sqrt(),
            attn_logit_softcapping: cfg.attn_logit_softcapping,
        })
    }

    fn forward(&self, x: &Tensor, rope: &RotaryEmbedding, seqlen_offset: usize) -> Result<Tensor> {
        let (b, seq_len, _) = x.dims3()?;

        let q = self.q_proj.forward(x)?;
        let k = self.k_proj.forward(x)?;
        let v = self.v_proj.forward(x)?;

        let q = q
            .reshape((b, seq_len, self.num_heads, self.head_dim))?
            .transpose(1, 2)?;
        let k = k
            .reshape((b, seq_len, self.num_kv_heads, self.head_dim))?
            .transpose(1, 2)?;
        let v = v
            .reshape((b, seq_len, self.num_kv_heads, self.head_dim))?
            .transpose(1, 2)?;

        let q = rope.apply(&q, seqlen_offset)?;
        let k = rope.apply(&k, seqlen_offset)?;

        let n_rep = self.num_heads / self.num_kv_heads;
        let k = repeat_kv(k, n_rep)?.contiguous()?;
        let v = repeat_kv(v, n_rep)?.contiguous()?;
        let q = q.contiguous()?;

        let attn_weights = (q.matmul(&k.transpose(2, 3)?)? * self.attn_scale)?;
        let attn_weights = softcap(&attn_weights, self.attn_logit_softcapping)?;

        // sliding_window(4096)가 학습 시퀀스 길이보다 항상 크므로 causal mask와
        // 슬라이딩 윈도우 마스크가 수학적으로 동일하다 — 별도 윈도우 마스킹이 불필요하다.
        let mask = causal_mask(seq_len, x.device())?;
        let attn_weights = attn_weights.broadcast_add(&mask)?;
        let attn_weights = candle_nn::ops::softmax_last_dim(&attn_weights)?;

        let attn_out = attn_weights.matmul(&v)?;
        let attn_out =
            attn_out
                .transpose(1, 2)?
                .reshape((b, seq_len, self.num_heads * self.head_dim))?;

        self.o_proj.forward(&attn_out)
    }
}

struct Gemma2Mlp {
    gate_proj: Linear,
    up_proj: Linear,
    down_proj: Linear,
}

impl Gemma2Mlp {
    fn load(cfg: &Gemma2Config, vb: VarBuilder) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let inter = cfg.intermediate_size;
        Ok(Self {
            gate_proj: Linear::new(vb.get((inter, hidden), "gate_proj.weight")?, None),
            up_proj: Linear::new(vb.get((inter, hidden), "up_proj.weight")?, None),
            down_proj: Linear::new(vb.get((hidden, inter), "down_proj.weight")?, None),
        })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        // Gemma2는 SwiGLU(silu)가 아니라 GeGLU(gelu_pytorch_tanh)를 쓴다.
        let gate = self.gate_proj.forward(x)?.gelu()?;
        let up = self.up_proj.forward(x)?;
        self.down_proj.forward(&(gate * up)?)
    }
}

struct Gemma2DecoderLayer {
    self_attn: Gemma2Attention,
    mlp: Gemma2Mlp,
    input_layernorm: RmsNorm,
    post_attention_layernorm: RmsNorm,
    pre_feedforward_layernorm: RmsNorm,
    post_feedforward_layernorm: RmsNorm,
}

impl Gemma2DecoderLayer {
    fn load(
        cfg: &Gemma2Config,
        vb: VarBuilder,
        lora_varmap: &VarMap,
        lora_rank: usize,
        lora_alpha: f64,
        layer_idx: usize,
        device: &Device,
    ) -> Result<Self> {
        Ok(Self {
            self_attn: Gemma2Attention::load(
                cfg,
                vb.pp("self_attn"),
                lora_varmap,
                lora_rank,
                lora_alpha,
                layer_idx,
                device,
            )?,
            mlp: Gemma2Mlp::load(cfg, vb.pp("mlp"))?,
            input_layernorm: RmsNorm::load(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("input_layernorm"))?,
            post_attention_layernorm: RmsNorm::load(
                cfg.hidden_size,
                cfg.rms_norm_eps,
                vb.pp("post_attention_layernorm"),
            )?,
            pre_feedforward_layernorm: RmsNorm::load(
                cfg.hidden_size,
                cfg.rms_norm_eps,
                vb.pp("pre_feedforward_layernorm"),
            )?,
            post_feedforward_layernorm: RmsNorm::load(
                cfg.hidden_size,
                cfg.rms_norm_eps,
                vb.pp("post_feedforward_layernorm"),
            )?,
        })
    }

    fn forward(&self, x: &Tensor, rope: &RotaryEmbedding, seqlen_offset: usize) -> Result<Tensor> {
        let residual = x;
        let h = self.input_layernorm.forward(x)?;
        let h = self.self_attn.forward(&h, rope, seqlen_offset)?;
        let h = self.post_attention_layernorm.forward(&h)?;
        let x = (residual + h)?;

        let residual = &x;
        let h = self.pre_feedforward_layernorm.forward(&x)?;
        let h = self.mlp.forward(&h)?;
        let h = self.post_feedforward_layernorm.forward(&h)?;
        residual + h
    }
}

pub struct Gemma2Model {
    embed_tokens: Embedding,
    layers: Vec<Gemma2DecoderLayer>,
    norm: RmsNorm,
    lm_head: Linear,
    rope: RotaryEmbedding,
    hidden_size: usize,
    final_logit_softcapping: f64,
}

impl Gemma2Model {
    pub fn load(
        cfg: &Gemma2Config,
        vb: VarBuilder,
        lora_varmap: &VarMap,
        lora_rank: usize,
        lora_alpha: f64,
        device: &Device,
    ) -> Result<Self> {
        let embed_tokens = Embedding::new(
            vb.get(
                (cfg.vocab_size, cfg.hidden_size),
                "model.embed_tokens.weight",
            )?,
            cfg.hidden_size,
        );

        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        let layers_vb = vb.pp("model.layers");
        for i in 0..cfg.num_hidden_layers {
            layers.push(Gemma2DecoderLayer::load(
                cfg,
                layers_vb.pp(i),
                lora_varmap,
                lora_rank,
                lora_alpha,
                i,
                device,
            )?);
        }

        let norm = RmsNorm::load(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("model.norm"))?;

        // Gemma 계열은 항상 임베딩과 lm_head를 공유(tie)한다.
        let lm_head_weight = vb
            .get((cfg.vocab_size, cfg.hidden_size), "lm_head.weight")
            .or_else(|_| {
                vb.get(
                    (cfg.vocab_size, cfg.hidden_size),
                    "model.embed_tokens.weight",
                )
            })?;
        let lm_head = Linear::new(lm_head_weight, None);

        let rope = RotaryEmbedding::new(
            cfg.head_dim,
            cfg.rope_theta,
            cfg.max_position_embeddings,
            device,
        )?;

        Ok(Self {
            embed_tokens,
            layers,
            norm,
            lm_head,
            rope,
            hidden_size: cfg.hidden_size,
            final_logit_softcapping: cfg.final_logit_softcapping,
        })
    }

    pub fn forward(&self, input_ids: &Tensor) -> Result<Tensor> {
        let mut x = self.embed_tokens.forward(input_ids)?;
        // Gemma 계열은 임베딩 직후 sqrt(hidden_size)로 스케일링한다(공식 스펙).
        let embed_scale = (self.hidden_size as f64).sqrt();
        x = (x * embed_scale)?;

        for layer in &self.layers {
            x = layer.forward(&x, &self.rope, 0)?;
        }
        let x = self.norm.forward(&x)?;
        let logits = self.lm_head.forward(&x)?;
        softcap(&logits, self.final_logit_softcapping)
    }
}
