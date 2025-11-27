use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub models_dir: PathBuf,
    pub data_dir: PathBuf,
    pub device_preference: DevicePreference,
    pub auto_start: bool,
    pub keep_server_running: bool,
    #[serde(default = "default_resource_mode")]
    pub resource_mode: ResourceMode,
    #[serde(default = "default_context_length")]
    pub default_context_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DevicePreference {
    Auto,
    GPU,
    CPU,
    NPU,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceMode {
    Strict,
    Loose,
}

fn default_resource_mode() -> ResourceMode {
    ResourceMode::Strict
}

fn default_context_length() -> u64 {
    4096
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = Self::default_data_dir();
        Self {
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
            models_dir: data_dir.join("models"),
            data_dir: data_dir.clone(),
            device_preference: DevicePreference::Auto,
            auto_start: false,
            keep_server_running: false,
            resource_mode: ResourceMode::Strict,
            default_context_length: 4096,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::create_dir_all(&self.models_dir)?;
        fs::create_dir_all(&self.data_dir)?;

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;

        Ok(())
    }

    pub fn config_path() -> PathBuf {
        if cfg!(target_os = "linux") {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("capi/config.json")
        } else if cfg!(target_os = "windows") {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
                .join("capi\\config.json")
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/Library/Application Support"))
                .join("capi/config.json")
        }
    }

    pub fn default_data_dir() -> PathBuf {
        if cfg!(target_os = "linux") {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("~/.local/share"))
                .join("capi")
        } else if cfg!(target_os = "windows") {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
                .join("capi")
        } else {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("~/Library/Application Support"))
                .join("capi")
        }
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("capi.db")
    }

    pub fn server_url(&self) -> String {
        format!("http://{}:{}", self.server_host, self.server_port)
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
