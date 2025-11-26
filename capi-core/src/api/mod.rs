pub mod chat;
mod embeddings;
mod models;

use axum::{Router, routing::post};
pub use chat::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(chat::completions))
        .route("/v1/completions", post(chat::completions_legacy))
        .route("/v1/embeddings", post(embeddings::create))
        .route("/v1/models", axum::routing::get(models::list))
        .with_state(state)
}
