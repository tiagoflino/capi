use super::{DeviceInfo, DeviceType};
use crate::config::DevicePreference;

pub fn select_best_device(devices: &[DeviceInfo], preference: &DevicePreference) -> Option<String> {
    match preference {
        DevicePreference::Auto => select_fastest_device(devices),
        DevicePreference::GPU => find_device_by_type(devices, DeviceType::GPU),
        DevicePreference::NPU => find_device_by_type(devices, DeviceType::NPU),
        DevicePreference::CPU => find_device_by_type(devices, DeviceType::CPU),
    }
}

fn select_fastest_device(devices: &[DeviceInfo]) -> Option<String> {
    let priority = vec![DeviceType::GPU, DeviceType::NPU, DeviceType::CPU];

    for device_type in priority {
        if let Some(name) = find_device_by_type(devices, device_type) {
            return Some(name);
        }
    }

    devices.first().map(|d| d.name.clone())
}

fn find_device_by_type(devices: &[DeviceInfo], device_type: DeviceType) -> Option<String> {
    devices
        .iter()
        .find(|d| d.device_type == device_type && d.available)
        .map(|d| d.name.clone())
}
