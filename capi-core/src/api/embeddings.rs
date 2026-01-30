use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct EmbeddingRequest {
    pub model: Option<String>,
    pub input: EmbeddingInput,
}

#[derive(Deserialize)]
#[allow(dead_code)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
}

#[derive(Serialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

pub async fn create(
    Json(_payload): Json<EmbeddingRequest>,
) -> impl IntoResponse {
    Json(EmbeddingResponse {
        object: "list".to_string(),
        data: vec![],
        model: "default".to_string(),
    })
}
