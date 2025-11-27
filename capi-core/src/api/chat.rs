use axum::{
    Json,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    extract::State,
    http::StatusCode,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::convert::Infallible;

use crate::model_manager::Registry;
use crate::InferenceSession;
use std::collections::HashMap;

type ModelCache = Arc<RwLock<HashMap<String, Arc<RwLock<InferenceSession>>>>>;

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<Registry>,
    pub model_cache: ModelCache,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Serialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_per_second: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to_first_token_ms: Option<f32>,
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
    state: AppState,
    payload: ChatCompletionRequest,
) -> anyhow::Result<ChatCompletionResponse> {
    let model_id = payload.model.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Model is required"))?;

    let mut cache = state.model_cache.write().await;

    let session = if let Some(cached_session) = cache.get(model_id) {
        Arc::clone(cached_session)
    } else {
        let model = state.registry.get_model(model_id)?
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        let config = crate::Config::load()?;
        let devices = crate::hardware::detect_devices()?;
        let device = crate::hardware::select_best_device(&devices, &config.device_preference)
            .unwrap_or_else(|| "CPU".to_string());

        let model_path = std::path::Path::new(&model.path);
        let loaded_session = crate::InferenceSession::load(model_path, &device)?;
        let session_arc = Arc::new(RwLock::new(loaded_session));

        cache.insert(model_id.clone(), Arc::clone(&session_arc));
        session_arc
    };

    drop(cache);

    let session_guard = session.read().await;

    let conversation: String = payload.messages.iter()
        .map(|m| format!("{}: {}",
            if m.role == "user" { "User" } else { "Assistant" },
            m.content
        ))
        .collect::<Vec<_>>()
        .join("\n");

    let full_prompt = conversation + "\nAssistant:";
    let max_tokens = payload.max_tokens.unwrap_or(4096);

    let (response_text, metrics) = session_guard.generate_with_metrics(&full_prompt, max_tokens)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    Ok(ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: timestamp,
        model: model_id.clone(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: response_text,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Some(Usage {
            prompt_tokens: metrics.num_input_tokens,
            completion_tokens: metrics.num_output_tokens,
            total_tokens: metrics.num_input_tokens + metrics.num_output_tokens,
            tokens_per_second: Some(metrics.tokens_per_second),
            time_to_first_token_ms: Some(metrics.time_to_first_token_ms),
        }),
    })
}

fn create_streaming_response(
    state: AppState,
    payload: ChatCompletionRequest,
) -> impl Stream<Item = Result<Event, Infallible>> {
    use async_stream::stream;
    use tokio::sync::mpsc;

    stream! {
        let model_id = match payload.model.as_ref() {
            Some(id) => id.clone(),
            None => return,
        };

        let mut cache = state.model_cache.write().await;

        let session = if let Some(cached_session) = cache.get(&model_id) {
            Arc::clone(cached_session)
        } else {
            let model = match state.registry.get_model(&model_id) {
                Ok(Some(m)) => m,
                _ => return,
            };

            let config = match crate::Config::load() {
                Ok(c) => c,
                Err(_) => return,
            };

            let devices = match crate::hardware::detect_devices() {
                Ok(d) => d,
                Err(_) => return,
            };

            let device = crate::hardware::select_best_device(&devices, &config.device_preference)
                .unwrap_or_else(|| "CPU".to_string());

            let model_path = std::path::Path::new(&model.path);
            let loaded_session = match crate::InferenceSession::load(model_path, &device) {
                Ok(s) => s,
                Err(_) => return,
            };

            let session_arc = Arc::new(RwLock::new(loaded_session));
            cache.insert(model_id.clone(), Arc::clone(&session_arc));
            session_arc
        };

        drop(cache);

        let session_guard = session.read().await;

        let conversation: String = payload.messages.iter()
            .map(|m| format!("{}: {}", if m.role == "user" { "User" } else { "Assistant" }, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let full_prompt = conversation + "\nAssistant:";
        let max_tokens = payload.max_tokens.unwrap_or(4096);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
        let id_clone = id.clone();
        let model_clone = model_id.clone();

        let (tx, mut rx) = mpsc::unbounded_channel();

        let session_clone = Arc::clone(&session);
        let prompt_clone = full_prompt.clone();

        tokio::task::spawn_blocking(move || {
            let session_guard = session_clone.blocking_read();
            session_guard.generate_stream(&prompt_clone, max_tokens, move |token| {
                tx.send(token.to_string()).ok();
                true
            })
        });

        let mut is_first = true;

        while let Some(token) = rx.recv().await {
            let chunk = ChatCompletionChunk {
                id: id_clone.clone(),
                object: "chat.completion.chunk".to_string(),
                created: timestamp,
                model: model_clone.clone(),
                choices: vec![ChunkChoice {
                    index: 0,
                    delta: Delta {
                        role: if is_first { Some("assistant".to_string()) } else { None },
                        content: Some(token),
                    },
                    finish_reason: None,
                }],
            };

            is_first = false;

            let json = serde_json::to_string(&chunk).unwrap();
            yield Ok(Event::default().data(json));
        }

        drop(session_guard);

        let final_chunk = ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: timestamp,
            model: model_id.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        let json = serde_json::to_string(&final_chunk).unwrap();
        yield Ok(Event::default().data(json));
    }
}

pub async fn completions_legacy(
    state: State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Response {
    completions(state, Json(payload)).await
}
