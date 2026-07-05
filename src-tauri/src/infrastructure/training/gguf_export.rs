use candle_core::quantized::{gguf_file, GgmlDType, QTensor};
use candle_nn::VarMap;
use std::io::{BufWriter, Write};

use super::config::Qwen2Config;

const LORA_PROJECTIONS: &[(&str, &str)] = &[("q_proj", "attn_q"), ("v_proj", "attn_v")];

pub fn export_lora_to_gguf(
    varmap: &VarMap,
    cfg: &Qwen2Config,
    alpha: f64,
    output_path: &std::path::Path,
) -> anyhow::Result<()> {
    let data = varmap
        .data()
        .lock()
        .map_err(|e| anyhow::anyhow!("LoRA VarMap 락 실패: {e}"))?;

    let mut tensors_owned: Vec<(String, QTensor)> = Vec::new();
    for layer_idx in 0..cfg.num_hidden_layers {
        for (candle_name, gguf_name) in LORA_PROJECTIONS {
            for suffix in ["lora_a", "lora_b"] {
                let candle_key = format!("layer{layer_idx}.{candle_name}.{suffix}");
                let var = data
                    .get(&candle_key)
                    .ok_or_else(|| anyhow::anyhow!("LoRA 텐서를 찾을 수 없습니다: {candle_key}"))?;
                let qtensor = QTensor::quantize(var.as_tensor(), GgmlDType::F32)?;
                let gguf_key = format!("blk.{layer_idx}.{gguf_name}.weight.{suffix}");
                tensors_owned.push((gguf_key, qtensor));
            }
        }
    }
    drop(data);

    let metadata_owned: Vec<(&str, gguf_file::Value)> = vec![
        (
            "general.type",
            gguf_file::Value::String("adapter".to_string()),
        ),
        (
            "general.architecture",
            gguf_file::Value::String("qwen2".to_string()),
        ),
        ("adapter.type", gguf_file::Value::String("lora".to_string())),
        ("adapter.lora.alpha", gguf_file::Value::F32(alpha as f32)),
    ];
    let metadata_refs: Vec<(&str, &gguf_file::Value)> =
        metadata_owned.iter().map(|(k, v)| (*k, v)).collect();
    let tensor_refs: Vec<(&str, &QTensor)> =
        tensors_owned.iter().map(|(k, v)| (k.as_str(), v)).collect();

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    gguf_file::write(&mut writer, &metadata_refs, &tensor_refs)?;
    writer.flush()?;

    Ok(())
}
