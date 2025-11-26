use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::State;
use serde::Serialize;

struct AppData {
    db: Arc<capi_core::Database>,
    registry: Arc<capi_core::Registry>,
    downloader: capi_core::Downloader,
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
async fn get_config() -> Result<capi_core::Config, String> {
    capi_core::Config::load().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_config(config: capi_core::Config) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
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

    let app_data = AppData {
        db,
        registry,
        downloader,
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
            get_config,
            save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
