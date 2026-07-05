use candle_core::{DType, Device, Result, Tensor, D};
use candle_nn::{Embedding, Linear, Module, VarBuilder, VarMap};

use super::config::Qwen2Config;
use super::lora::LoraLinear;

struct RmsNorm {
    weight: Tensor,
    eps: f64,
}

impl RmsNorm {
    fn load(size: usize, eps: f64, vb: VarBuilder) -> Result<Self> {
        let weight = vb.get(size, "weight")?;
        Ok(Self { weight, eps })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let in_dtype = x.dtype();
        let x = x.to_dtype(DType::F32)?;
        let variance = x.sqr()?.mean_keepdim(D::Minus1)?;
        let x_normed = x.broadcast_div(&(variance + self.eps)?.sqrt()?)?;
        x_normed.to_dtype(in_dtype)?.broadcast_mul(&self.weight)
    }
}

struct RotaryEmbedding {
    cos: Tensor,
    sin: Tensor,
}

impl RotaryEmbedding {
    fn new(cfg: &Qwen2Config, device: &Device) -> Result<Self> {
        let head_dim = cfg.head_dim();
        let theta: Vec<f32> = (0..head_dim)
            .step_by(2)
            .map(|i| 1f32 / cfg.rope_theta.powf(i as f32 / head_dim as f32))
            .collect();
        let theta = Tensor::new(theta.as_slice(), device)?;
        let idx = Tensor::arange(0u32, cfg.max_position_embeddings as u32, device)?
            .to_dtype(DType::F32)?
            .reshape((cfg.max_position_embeddings, 1))?;
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

struct Attention {
    q_proj: LoraLinear,
    k_proj: Linear,
    v_proj: LoraLinear,
    o_proj: Linear,
    num_heads: usize,
    num_kv_heads: usize,
    head_dim: usize,
}

impl Attention {
    fn load(
        cfg: &Qwen2Config,
        vb: VarBuilder,
        lora_varmap: &VarMap,
        lora_rank: usize,
        lora_alpha: f64,
        layer_idx: usize,
        device: &Device,
    ) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let head_dim = cfg.head_dim();
        let kv_dim = cfg.num_key_value_heads * head_dim;

        let q_weight = vb.get((hidden, hidden), "q_proj.weight")?;
        let q_proj = LoraLinear::new(
            q_weight,
            hidden,
            hidden,
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
        let o_proj = Linear::new(vb.get((hidden, hidden), "o_proj.weight")?, None);

        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            num_heads: cfg.num_attention_heads,
            num_kv_heads: cfg.num_key_value_heads,
            head_dim,
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

        let scale = 1f64 / (self.head_dim as f64).sqrt();
        let attn_weights = (q.matmul(&k.transpose(2, 3)?)? * scale)?;

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

fn causal_mask(seq_len: usize, device: &Device) -> Result<Tensor> {
    let mask: Vec<f32> = (0..seq_len)
        .flat_map(|i| (0..seq_len).map(move |j| if j > i { f32::NEG_INFINITY } else { 0.0 }))
        .collect();
    Tensor::from_vec(mask, (1, 1, seq_len, seq_len), device)
}

struct Mlp {
    gate_proj: Linear,
    up_proj: Linear,
    down_proj: Linear,
}

impl Mlp {
    fn load(cfg: &Qwen2Config, vb: VarBuilder) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let inter = cfg.intermediate_size;
        Ok(Self {
            gate_proj: Linear::new(vb.get((inter, hidden), "gate_proj.weight")?, None),
            up_proj: Linear::new(vb.get((inter, hidden), "up_proj.weight")?, None),
            down_proj: Linear::new(vb.get((hidden, inter), "down_proj.weight")?, None),
        })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let gate = candle_nn::ops::silu(&self.gate_proj.forward(x)?)?;
        let up = self.up_proj.forward(x)?;
        self.down_proj.forward(&(gate * up)?)
    }
}

struct DecoderLayer {
    self_attn: Attention,
    mlp: Mlp,
    input_layernorm: RmsNorm,
    post_attention_layernorm: RmsNorm,
}

impl DecoderLayer {
    fn load(
        cfg: &Qwen2Config,
        vb: VarBuilder,
        lora_varmap: &VarMap,
        lora_rank: usize,
        lora_alpha: f64,
        layer_idx: usize,
        device: &Device,
    ) -> Result<Self> {
        Ok(Self {
            self_attn: Attention::load(
                cfg,
                vb.pp("self_attn"),
                lora_varmap,
                lora_rank,
                lora_alpha,
                layer_idx,
                device,
            )?,
            mlp: Mlp::load(cfg, vb.pp("mlp"))?,
            input_layernorm: RmsNorm::load(
                cfg.hidden_size,
                cfg.rms_norm_eps,
                vb.pp("input_layernorm"),
            )?,
            post_attention_layernorm: RmsNorm::load(
                cfg.hidden_size,
                cfg.rms_norm_eps,
                vb.pp("post_attention_layernorm"),
            )?,
        })
    }

    fn forward(&self, x: &Tensor, rope: &RotaryEmbedding, seqlen_offset: usize) -> Result<Tensor> {
        let residual = x;
        let x = self.input_layernorm.forward(x)?;
        let x = self.self_attn.forward(&x, rope, seqlen_offset)?;
        let x = (residual + x)?;

        let residual = &x;
        let h = self.post_attention_layernorm.forward(&x)?;
        let h = self.mlp.forward(&h)?;
        residual + h
    }
}

pub struct Qwen2Model {
    embed_tokens: Embedding,
    layers: Vec<DecoderLayer>,
    norm: RmsNorm,
    lm_head: Linear,
    rope: RotaryEmbedding,
}

impl Qwen2Model {
    pub fn load(
        cfg: &Qwen2Config,
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
            layers.push(DecoderLayer::load(
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

        let lm_head_weight = vb
            .get((cfg.vocab_size, cfg.hidden_size), "lm_head.weight")
            .or_else(|_| {
                vb.get(
                    (cfg.vocab_size, cfg.hidden_size),
                    "model.embed_tokens.weight",
                )
            })?;
        let lm_head = Linear::new(lm_head_weight, None);

        let rope = RotaryEmbedding::new(cfg, device)?;

        Ok(Self {
            embed_tokens,
            layers,
            norm,
            lm_head,
            rope,
        })
    }

    pub fn forward(&self, input_ids: &Tensor) -> Result<Tensor> {
        let mut x = self.embed_tokens.forward(input_ids)?;
        for layer in &self.layers {
            x = layer.forward(&x, &self.rope, 0)?;
        }
        let x = self.norm.forward(&x)?;
        self.lm_head.forward(&x)
    }
}
