mod device_detect;
mod priority;

pub use device_detect::{DeviceInfo, DeviceType, detect_devices};
pub use priority::select_best_device;
pub use crate::config::DevicePreference;
