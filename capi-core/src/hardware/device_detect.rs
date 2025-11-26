use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DeviceType {
    CPU,
    GPU,
    NPU,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub device_type: DeviceType,
    pub available: bool,
}

pub fn detect_devices() -> Result<Vec<DeviceInfo>> {
    let mut devices = vec![
        DeviceInfo {
            name: "CPU".to_string(),
            device_type: DeviceType::CPU,
            available: true,
        },
    ];

    if std::path::Path::new("/dev/dri").exists() {
        devices.push(DeviceInfo {
            name: "GPU".to_string(),
            device_type: DeviceType::GPU,
            available: true,
        });
    }

    Ok(devices)
}
