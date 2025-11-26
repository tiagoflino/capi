use axum::{Json, response::IntoResponse, extract::State, http::StatusCode};
use serde::Serialize;
use crate::api::chat::AppState;

#[derive(Serialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match state.registry.list_models() {
        Ok(models) => {
            let model_list = models.into_iter().map(|m| Model {
                id: m.id,
                object: "model".to_string(),
                created: m.created_at,
                owned_by: "user".to_string(),
            }).collect();

            Json(ModelList {
                object: "list".to_string(),
                data: model_list,
            }).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
