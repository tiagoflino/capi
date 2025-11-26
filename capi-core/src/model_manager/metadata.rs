use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_type: Option<String>,
    pub vocab_size: Option<u64>,
    pub hidden_size: Option<u64>,
    pub num_attention_heads: Option<u64>,
    pub num_hidden_layers: Option<u64>,
    pub max_position_embeddings: Option<u64>,
}

pub fn parse_config<P: AsRef<Path>>(config_path: P) -> Result<ModelMetadata> {
    let content = fs::read_to_string(config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    Ok(ModelMetadata {
        model_type: config.get("model_type").and_then(|v| v.as_str()).map(String::from),
        vocab_size: config.get("vocab_size").and_then(|v| v.as_u64()),
        hidden_size: config.get("hidden_size").and_then(|v| v.as_u64()),
        num_attention_heads: config.get("num_attention_heads").and_then(|v| v.as_u64()),
        num_hidden_layers: config.get("num_hidden_layers").and_then(|v| v.as_u64()),
        max_position_embeddings: config.get("max_position_embeddings").and_then(|v| v.as_u64()),
    })
}
