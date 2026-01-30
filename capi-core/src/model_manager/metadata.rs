// unused import removed
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_type: Option<String>,
    pub vocab_size: Option<u64>,
    pub hidden_size: Option<u64>,
    pub num_attention_heads: Option<u64>,
    pub num_hidden_layers: Option<u64>,
    pub max_position_embeddings: Option<u64>,
}

// parse_config removed as unused
