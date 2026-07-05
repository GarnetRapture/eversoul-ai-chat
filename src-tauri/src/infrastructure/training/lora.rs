use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{Init, Linear, Module, VarMap};

pub struct LoraLinear {
    base: Linear,
    lora_a: Tensor,
    lora_b: Tensor,
    scale: f64,
}

impl LoraLinear {
    pub fn new(
        base_weight: Tensor,
        in_features: usize,
        out_features: usize,
        rank: usize,
        alpha: f64,
        varmap: &VarMap,
        name_prefix: &str,
        device: &Device,
    ) -> Result<Self> {
        let base = Linear::new(base_weight, None);

        let lora_a = varmap.get(
            (rank, in_features),
            &format!("{name_prefix}.lora_a"),
            Init::Randn {
                mean: 0.0,
                stdev: 0.02,
            },
            DType::F32,
            device,
        )?;
        let lora_b = varmap.get(
            (out_features, rank),
            &format!("{name_prefix}.lora_b"),
            Init::Const(0.0),
            DType::F32,
            device,
        )?;

        Ok(Self {
            base,
            lora_a,
            lora_b,
            scale: alpha / rank as f64,
        })
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let base_out = self.base.forward(x)?;
        let lora_out = x
            .broadcast_matmul(&self.lora_a.t()?)?
            .broadcast_matmul(&self.lora_b.t()?)?;
        base_out + (lora_out * self.scale)?
    }
}

pub fn new_lora_varmap() -> VarMap {
    VarMap::new()
}

pub fn save_lora_weights(varmap: &VarMap, path: &std::path::Path) -> Result<()> {
    varmap.save(path)
}

pub fn load_lora_weights(varmap: &mut VarMap, path: &std::path::Path) -> Result<()> {
    varmap.load(path)
}
