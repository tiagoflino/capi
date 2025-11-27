mod device_detect;
mod priority;
mod resource_detect;
mod resource_validator;

pub use device_detect::{DeviceInfo, DeviceType, detect_devices};
pub use priority::select_best_device;
pub use resource_detect::{SystemResources, GpuResource, detect_system_resources};
pub use resource_validator::{ValidationResult, validate_model_load};
pub use crate::config::{DevicePreference, ResourceMode};
