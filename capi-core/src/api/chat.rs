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
    pub top_p: Option<f32>,
    pub top_k: Option<usize>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub repetition_penalty: Option<f32>,
    pub seed: Option<usize>,
    pub stop: Option<Vec<String>>,
    pub logprobs: Option<bool>,
    pub top_logprobs: Option<usize>,
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

    let mut session_guard = session.write().await;

    let conversation: String = payload.messages.iter()
        .map(|m| format!("{}: {}",
            if m.role == "user" { "User" } else { "Assistant" },
            m.content
        ))
        .collect::<Vec<_>>()
        .join("\n");

    let full_prompt = conversation + "\nAssistant:";
    let full_prompt = conversation + "\nAssistant:";
    let mut config = crate::inference::genai::GenerationConfig::new()?;
    if let Some(max_tokens) = payload.max_tokens {
        config.set_max_new_tokens(max_tokens)?;
    } else {
        config.set_max_new_tokens(4096)?;
    }

    if let Some(temp) = payload.temperature {
        config.set_temperature(temp)?;
    }
    if let Some(top_p) = payload.top_p {
        config.set_top_p(top_p)?;
    }
    if let Some(top_k) = payload.top_k {
        config.set_top_k(top_k)?;
    }
    if let Some(freq_pen) = payload.frequency_penalty {
        config.set_frequency_penalty(freq_pen)?;
    }
    if let Some(pres_pen) = payload.presence_penalty {
        config.set_presence_penalty(pres_pen)?;
    }
    if let Some(rep_pen) = payload.repetition_penalty {
        config.set_repetition_penalty(rep_pen)?;
    }
    if let Some(seed) = payload.seed {
        config.set_rng_seed(seed)?;
    }
    if let Some(stops) = payload.stop {
        let stops_ref: Vec<&str> = stops.iter().map(|s| s.as_str()).collect();
        config.set_stop_strings(&stops_ref)?;
    }
    if let Some(logprobs) = payload.logprobs {
        if logprobs {
            config.set_logprobs(payload.top_logprobs.unwrap_or(1))?;
        }
    }

    let (response_text, metrics) = session_guard.generate_with_metrics(&full_prompt, &config)?;

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

        // We don't need to hold the lock here, as spawn_blocking will acquire its own.
        // Holding it here would cause a deadlock with blocking_write below.
        
        let conversation: String = payload.messages.iter()
            .map(|m| format!("{}: {}", if m.role == "user" { "User" } else { "Assistant" }, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let full_prompt = conversation + "\nAssistant:";
        let full_prompt = conversation + "\nAssistant:";
        
        // Build configuration
        let mut config = match crate::inference::genai::GenerationConfig::new() {
            Ok(c) => c,
            Err(_) => return,
        };

        if let Some(max_tokens) = payload.max_tokens {
            let _ = config.set_max_new_tokens(max_tokens);
        } else {
            let _ = config.set_max_new_tokens(4096);
        }

        if let Some(temp) = payload.temperature {
            let _ = config.set_temperature(temp);
        }
        if let Some(top_p) = payload.top_p {
            let _ = config.set_top_p(top_p);
        }
        if let Some(top_k) = payload.top_k {
            let _ = config.set_top_k(top_k);
        }
        if let Some(freq_pen) = payload.frequency_penalty {
            let _ = config.set_frequency_penalty(freq_pen);
        }
        if let Some(pres_pen) = payload.presence_penalty {
            let _ = config.set_presence_penalty(pres_pen);
        }
        if let Some(rep_pen) = payload.repetition_penalty {
            let _ = config.set_repetition_penalty(rep_pen);
        }
        if let Some(seed) = payload.seed {
            let _ = config.set_rng_seed(seed);
        }
        if let Some(stops) = payload.stop {
            let stops_ref: Vec<&str> = stops.iter().map(|s| s.as_str()).collect();
            let _ = config.set_stop_strings(&stops_ref);
        }
        if let Some(logprobs) = payload.logprobs {
            if logprobs {
                let _ = config.set_logprobs(payload.top_logprobs.unwrap_or(1));
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
        let id_clone = id.clone();
        let model_clone = model_id.clone();

        let (tx, mut rx) = mpsc::unbounded_channel();

        let session_clone = Arc::clone(&session);
        let prompt_clone = full_prompt.clone();

        // SAFETY: GenerationConfig is Send + Sync and copied into the C++ pipeline
        // We need to move it into the closure, but GenerateConfig is not Clone.
        // However, we create a new config for each request, so we can just move it.
        // The issue is that spawn_blocking requires 'static, so we need to be careful.
        // Actually, config is moved into spawn_blocking, so it's fine.
        
        tokio::task::spawn_blocking(move || {
            let mut session_guard = session_clone.blocking_write();
            // We need to pass the config by reference to the session method
            session_guard.generate_stream(&prompt_clone, &config, move |token| {
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

            if let Ok(json) = serde_json::to_string(&chunk) {
                yield Ok(Event::default().data(json));
            }
        }

        // drop(session_guard); - removed as not acquired


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

        if let Ok(json) = serde_json::to_string(&final_chunk) {
            yield Ok(Event::default().data(json));
        }
    }
}

pub async fn completions_legacy(
    state: State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Response {
    completions(state, Json(payload)).await
}
