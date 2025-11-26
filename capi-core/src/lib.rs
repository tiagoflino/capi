pub mod api;
pub mod inference;
pub mod model_manager;
pub mod hardware;
pub mod config;
pub mod db;

pub use api::{create_router, AppState};
pub use config::Config;
pub use db::Database;
pub use inference::{InferenceSession, InferenceMetrics};
pub use model_manager::{Registry, Downloader, ModelInfo, HuggingFaceModel, ModelData, FileInfo};
pub use hardware::{detect_devices, select_best_device, DeviceInfo, DeviceType};
