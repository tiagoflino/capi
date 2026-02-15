use anyhow::{Result, Context};
use clap::Parser;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,

    /// Model path to load immediately
    #[arg(short, long)]
    model: Option<String>,

    /// Device to use for inference (CPU, GPU, NPU)
    #[arg(short, long)]
    device: Option<String>,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse args immediately
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    tracing::info!("Starting capi-server with args: {:?}", args);

    // Load config but allow args to override
    let mut config = capi_core::Config::load().unwrap_or_default();
    
    // Override config with CLI args
    if let Some(port) = args.port {
        config.server.port = port;
    }
    
    // Initialize core components
    let db = Arc::new(capi_core::Database::open(config.database_path())?);
    let registry = Arc::new(capi_core::Registry::new(db.clone()));
    let model_cache = Arc::new(RwLock::new(HashMap::new()));
    
    // Pre-load model if specified
    if let Some(model_path_str) = args.model {
        tracing::info!("Pre-loading model from: {}", model_path_str);
        
        // Determine device
        let device = args.device.unwrap_or_else(|| {
            // If no device specified, try to detect best
            match capi_core::hardware::detect_devices() {
                Ok(devices) => capi_core::hardware::select_best_device(&devices, &config.device_preference)
                    .unwrap_or_else(|| "CPU".to_string()),
                Err(_) => "CPU".to_string(),
            }
        });
        
        tracing::info!("Loading model on device: {}", device);
        
        let model_path = std::path::Path::new(&model_path_str);
        // The model ID can be the filename or just "default" for single-model mode
        let model_id = model_path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "default".to_string());
            
        match capi_core::InferenceSession::load(model_path, &device) {
            Ok(session) => {
                let session_arc = Arc::new(RwLock::new(session));
                model_cache.write().await.insert(model_id.clone(), session_arc);
                tracing::info!("Model '{}' loaded successfully", model_id);
            }
            Err(e) => {
                tracing::error!("Failed to load model: {}", e);
                // We don't panic here, we just start without the model loaded
            }
        }
    }

    let state = capi_core::AppState {
        registry,
        model_cache,
    };

    let app = capi_core::create_router(state);
    
    // Construct address from host and port
    let addr_str = format!("{}:{}", args.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr_str).await
        .context(format!("Failed to bind to address: {}", addr_str))?;

    // CRITICAL: Jan plugin system looks for this exact string to know the server is ready
    let url = format!("http://{}", addr_str);
    println!("Server is listening on {}", url);
    tracing::info!("Server is listening on {}", url);

    axum::serve(listener, app).await?;

    Ok(())
}
