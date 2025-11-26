use axum::{
    Json,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    extract::State,
    http::StatusCode,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::convert::Infallible;

use crate::model_manager::Registry;

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<Registry>,
}

#[derive(Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
    pub stream: Option<bool>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Serialize)]
pub struct Choice {
    pub index: usize,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Serialize)]
pub struct ChunkChoice {
    pub index: usize,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Serialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

pub async fn completions(
    State(state): State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Response {
    let stream = payload.stream.unwrap_or(false);

    if stream {
        let stream = create_streaming_response(state, payload);
        Sse::new(stream).into_response()
    } else {
        match create_non_streaming_response(state, payload).await {
            Ok(response) => Json(response).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

async fn create_non_streaming_response(
    _state: AppState,
    payload: ChatCompletionRequest,
) -> anyhow::Result<ChatCompletionResponse> {
    let prompt = payload.messages.last()
        .map(|m| m.content.as_str())
        .unwrap_or("");

    let response_text = format!("Echo: {}", prompt);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    Ok(ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: timestamp,
        model: payload.model.unwrap_or_else(|| "default".to_string()),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: response_text,
            },
            finish_reason: "stop".to_string(),
        }],
    })
}

fn create_streaming_response(
    _state: AppState,
    payload: ChatCompletionRequest,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let model = payload.model.unwrap_or_else(|| "default".to_string());
    let prompt = payload.messages.last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let tokens = vec![
        "Hello".to_string(),
        " from".to_string(),
        " Capi".to_string(),
        "!".to_string(),
    ];

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let id_clone = id.clone();
    let model_clone = model.clone();

    let token_events: Vec<_> = tokens.into_iter().enumerate().map(move |(i, token)| {
        let chunk = ChatCompletionChunk {
            id: id_clone.clone(),
            object: "chat.completion.chunk".to_string(),
            created: timestamp,
            model: model_clone.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: if i == 0 { Some("assistant".to_string()) } else { None },
                    content: Some(token),
                },
                finish_reason: None,
            }],
        };

        let json = serde_json::to_string(&chunk).unwrap();
        Ok(Event::default().data(json))
    }).collect();

    let final_chunk = {
        let chunk = ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: timestamp,
            model: model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        let json = serde_json::to_string(&chunk).unwrap();
        Ok(Event::default().data(json))
    };

    stream::iter(token_events.into_iter().chain(std::iter::once(final_chunk)))
}

pub async fn completions_legacy(
    state: State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Response {
    completions(state, Json(payload)).await
}
