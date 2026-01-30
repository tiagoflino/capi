use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};
use serde::Serialize;

struct AppData {
    #[allow(dead_code)]
    db: Arc<capi_core::Database>,
    registry: Arc<capi_core::Registry>,
    downloader: capi_core::Downloader,
    sessions: Arc<Mutex<std::collections::HashMap<String, capi_core::InferenceSession>>>,
}

struct ServerState {
    process: Arc<Mutex<Option<Child>>>,
}

#[tauri::command]
async fn start_server(state: State<'_, ServerState>) -> Result<(), String> {
    let server_exe = get_server_executable_path();

    let child = Command::new(server_exe)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut process = state.process.lock()
        .map_err(|_| "Failed to lock process")?;
    *process = Some(child);

    Ok(())
}

#[tauri::command]
async fn stop_server(state: State<'_, ServerState>) -> Result<(), String> {
    let mut process = state.process.lock()
        .map_err(|_| "Failed to lock process")?;

    if let Some(mut child) = process.take() {
        child.kill().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[derive(Serialize)]
struct ServerStatus {
    running: bool,
    url: String,
    host: String,
    port: u16,
}

#[tauri::command]
async fn get_server_status(state: State<'_, ServerState>) -> Result<ServerStatus, String> {
    let process = state.process.lock()
        .map_err(|_| "Failed to lock process")?;

    let running = process.as_ref()
        .and_then(|c| c.id().try_into().ok())
        .map(|_: u32| true)
        .unwrap_or(false);

    let config = capi_core::Config::load()
        .map_err(|e| e.to_string())?;

    Ok(ServerStatus {
        running,
        url: config.server_url(),
        host: config.server_host.clone(),
        port: config.server_port,
    })
}

#[derive(Serialize)]
struct HardwareStatus {
    available_devices: Vec<capi_core::hardware::DeviceInfo>,
    selected_device: Option<String>,
}

#[tauri::command]
async fn get_hardware_status() -> Result<HardwareStatus, String> {
    let devices = capi_core::detect_devices()
        .map_err(|e| e.to_string())?;

    let config = capi_core::Config::load()
        .map_err(|e| e.to_string())?;

    let selected = capi_core::select_best_device(&devices, &config.device_preference);

    Ok(HardwareStatus {
        available_devices: devices,
        selected_device: selected,
    })
}

#[tauri::command]
async fn search_models(
    query: String,
    state: State<'_, AppData>,
) -> Result<Vec<capi_core::model_manager::HuggingFaceModel>, String> {
    state.downloader
        .search_models(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_models(
    state: State<'_, AppData>,
) -> Result<Vec<capi_core::db::ModelRecord>, String> {
    state.registry
        .list_models()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_model(
    model_id: String,
    state: State<'_, AppData>,
) -> Result<(), String> {
    let config = capi_core::Config::load().map_err(|e| e.to_string())?;
    let model_path = config.models_dir.join(&model_id.replace("/", "_"));

    state.downloader
        .download_model(&model_id, &model_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_model(
    model_id: String,
    state: State<'_, AppData>,
) -> Result<(), String> {
    state.registry
        .remove_model(&model_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn find_quantized_versions(
    base_model_id: String,
    state: State<'_, AppData>,
) -> Result<Vec<capi_core::HuggingFaceModel>, String> {
    state.downloader
        .find_quantized_versions(&base_model_id)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct FileInfoResponse {
    name: String,
    size: Option<u64>,
}

#[tauri::command]
async fn fetch_model_files(
    model_id: String,
    state: State<'_, AppData>,
) -> Result<Vec<FileInfoResponse>, String> {
    let data = state.downloader
        .fetch_model_data(&model_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(data.files_with_size.into_iter().map(|f| FileInfoResponse {
        name: f.name,
        size: f.size,
    }).collect())
}

#[derive(Serialize, Clone)]
struct DownloadProgress {
    current: u64,
    total: u64,
    percent: f64,
}

#[tauri::command]
async fn download_specific_file(
    app: tauri::AppHandle,
    model_id: String,
    filename: String,
    state: State<'_, AppData>,
) -> Result<(), String> {
    let config = capi_core::Config::load().map_err(|e| e.to_string())?;
    let safe_name = model_id.replace("/", "_");
    let model_path = config.models_dir.join(&safe_name);

    std::fs::create_dir_all(&model_path).map_err(|e| e.to_string())?;

    let url = format!("https://huggingface.co/{}/resolve/main/{}", model_id, filename);

    state.downloader
        .download_file_with_progress(&url, &model_path.join(&filename), move |current, total| {
            if total > 0 {
                let percent = current as f64 / total as f64 * 100.0;
                app.emit("download-progress", DownloadProgress {
                    current,
                    total,
                    percent,
                }).ok();
            }
        })
        .await
        .map_err(|e| e.to_string())?;

    // Register model
    let file_path = model_path.join(&filename);
    let file_size = std::fs::metadata(&file_path).ok().map(|m| m.len() as i64);
    let estimated_memory = file_size.map(|s| (s as f64 * 1.5) as i64);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let model_record = capi_core::db::ModelRecord {
        id: safe_name.clone(),
        name: model_id.split('/').last().unwrap_or(&model_id).to_string(),
        path: file_path.to_string_lossy().to_string(),
        size_bytes: file_size,
        quantization: Some(filename.clone()),
        context_length: None,
        created_at: timestamp,
        last_used: None,
        estimated_memory_bytes: estimated_memory,
        context_override: None,
    };

    state.registry.add_model(model_record).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_config() -> Result<capi_core::Config, String> {
    capi_core::Config::load().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_config(config: capi_core::Config) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct SystemResourcesResponse {
    total_ram_bytes: u64,
    available_ram_bytes: u64,
    cpu_usage_percent: f32,
    selected_device: Option<String>,
    gpu_resources: Vec<GpuResourceResponse>,
}

#[derive(Serialize)]
struct GpuResourceResponse {
    name: String,
    total_vram_bytes: u64,
    available_vram_bytes: u64,
    usage_percent: f32,
    frequency_mhz: u32,
    max_frequency_mhz: u32,
}

#[tauri::command]
async fn load_model_direct(
    model_id: String,
    state: State<'_, AppData>,
) -> Result<String, String> {
    // Unload all previous models first
    {
        let mut sessions = state.sessions.lock()
            .map_err(|_| "Failed to acquire sessions lock".to_string())?;
        sessions.clear();
    }

    let model = state.registry.get_model(&model_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Model not found: {}", model_id))?;

    let config = capi_core::Config::load().map_err(|e| e.to_string())?;
    let devices = capi_core::detect_devices().map_err(|e| e.to_string())?;
    let device = capi_core::select_best_device(&devices, &config.device_preference)
        .unwrap_or_else(|| "CPU".to_string());

    let model_path = std::path::Path::new(&model.path);
    println!("Loading model {} from {:?} on device {}", model_id, model_path, device);

    let mut session = capi_core::InferenceSession::load_with_lock(model_path, &device, &model_id)
        .map_err(|e| {
            println!("Error loading model into session: {}", e);
            e.to_string()
        })?;

    println!("Starting chat for session...");
    session.start_chat().map_err(|e| {
        println!("Error starting chat: {}", e);
        e.to_string()
    })?;
    println!("Model {} successfully loaded", model_id);

    let mut sessions = state.sessions.lock()
        .map_err(|_| "Failed to acquire sessions lock".to_string())?;

    sessions.insert(model_id.clone(), session);

    Ok(format!("Model {} loaded on {}", model_id, device))
}

#[derive(Serialize, Clone)]
struct ChatToken {
    token: String,
}

#[derive(Serialize, Clone)]
struct ChatMetrics {
    tokens_per_second: f32,
    time_to_first_token_ms: f32,
    num_output_tokens: usize,
}

#[tauri::command]
async fn chat_direct(
    app: tauri::AppHandle,
    model_id: String,
    prompt: String,
    state: State<'_, AppData>,
) -> Result<ChatMetrics, String> {
    let mut sessions = state.sessions.lock()
        .map_err(|_| "Failed to acquire sessions lock".to_string())?;

    let session = sessions.get_mut(&model_id)
        .ok_or_else(|| format!("Model {} not loaded. Load it first.", model_id))?;

    let (_response, metrics) = session.generate_stream(&prompt, 4096, move |token| {
        app.emit("chat-token", ChatToken { token: token.to_string() }).ok();
        true
    }).map_err(|e| e.to_string())?;

    Ok(ChatMetrics {
        tokens_per_second: metrics.tokens_per_second,
        time_to_first_token_ms: metrics.time_to_first_token_ms,
        num_output_tokens: metrics.num_output_tokens,
    })
}

#[tauri::command]
async fn preload_model(model_id: String) -> Result<String, String> {
    let config = capi_core::Config::load().map_err(|e| e.to_string())?;
    let url = format!("{}/v1/chat/completions", config.server_url());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))  // 5 minutes for large models
        .build()
        .map_err(|e| e.to_string())?;

    // Make a minimal request to trigger model loading
    let body = serde_json::json!({
        "model": model_id,
        "messages": [{"role": "user", "content": "test"}],
        "max_tokens": 1,
        "stream": false,
    });

    match client.post(&url)
        .json(&body)
        .send()
        .await
    {
        Ok(_) => Ok("Model loaded".to_string()),
        Err(e) => Err(format!("Failed to preload model: {}", e)),
    }
}

#[tauri::command]
async fn get_system_resources() -> Result<SystemResourcesResponse, String> {
    let resources = capi_core::detect_system_resources()
        .map_err(|e| e.to_string())?;

    let config = capi_core::Config::load().map_err(|e| e.to_string())?;
    let devices = capi_core::detect_devices().map_err(|e| e.to_string())?;
    let selected_device = capi_core::select_best_device(&devices, &config.device_preference);

    let cpu_usage = get_cpu_usage();

    Ok(SystemResourcesResponse {
        total_ram_bytes: resources.total_ram_bytes,
        available_ram_bytes: resources.available_ram_bytes,
        cpu_usage_percent: cpu_usage,
        selected_device,
        gpu_resources: resources.gpu_resources.iter().map(|gpu| GpuResourceResponse {
            name: gpu.name.clone(),
            total_vram_bytes: gpu.total_vram_bytes,
            available_vram_bytes: gpu.available_vram_bytes,
            usage_percent: gpu.usage_percent,
            frequency_mhz: gpu.frequency_mhz,
            max_frequency_mhz: gpu.max_frequency_mhz,
        }).collect(),
    })
}

fn get_cpu_usage() -> f32 {
    #[cfg(target_os = "linux")]
    {
        use std::thread;
        use std::time::Duration;

        if let Ok(stat1) = std::fs::read_to_string("/proc/stat") {
            if let Some(line1) = stat1.lines().next() {
                let parts1: Vec<&str> = line1.split_whitespace().collect();
                if parts1.len() > 4 && parts1[0] == "cpu" {
                    thread::sleep(Duration::from_millis(100));

                    if let Ok(stat2) = std::fs::read_to_string("/proc/stat") {
                        if let Some(line2) = stat2.lines().next() {
                            let parts2: Vec<&str> = line2.split_whitespace().collect();
                            if parts2.len() > 4 && parts2[0] == "cpu" {
                                let idle1: f32 = parts1[4].parse().unwrap_or(0.0);
                                let idle2: f32 = parts2[4].parse().unwrap_or(0.0);

                                let total1: f32 = parts1[1..].iter().filter_map(|s| s.parse::<f32>().ok()).sum();
                                let total2: f32 = parts2[1..].iter().filter_map(|s| s.parse::<f32>().ok()).sum();

                                let total_diff = total2 - total1;
                                let idle_diff = idle2 - idle1;

                                if total_diff > 0.0 {
                                    return ((total_diff - idle_diff) / total_diff * 100.0).min(100.0).max(0.0);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    0.0
}

fn get_server_executable_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    #[cfg(windows)]
    return exe_dir.join("capi-server.exe");

    #[cfg(not(windows))]
    exe_dir.join("capi-server")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = capi_core::Config::load().expect("Failed to load config");
    let db = Arc::new(
        capi_core::Database::open(config.database_path())
            .expect("Failed to open database")
    );

    let registry = Arc::new(capi_core::Registry::new(db.clone()));
    let downloader = capi_core::Downloader::new();
    let sessions = Arc::new(Mutex::new(std::collections::HashMap::new()));

    let app_data = AppData {
        db,
        registry,
        downloader,
        sessions,
    };

    let server_state = ServerState {
        process: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_data)
        .manage(server_state)
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            get_server_status,
            get_hardware_status,
            search_models,
            list_models,
            download_model,
            remove_model,
            find_quantized_versions,
            fetch_model_files,
            download_specific_file,
            get_config,
            save_config,
            get_system_resources,
            preload_model,
            load_model_direct,
            chat_direct,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
