use anyhow::Result;
use super::{SystemResources, ResourceMode};

#[derive(Debug, Clone)]
pub enum ValidationResult {
    Sufficient,
    Warning { message: String },
    Insufficient { message: String },
}

pub fn validate_model_load(
    estimated_memory: u64,
    device: &str,
    system_resources: &SystemResources,
    mode: &ResourceMode,
) -> Result<ValidationResult> {
    let (available, resource_name) = if device.to_uppercase().contains("GPU") {
        if let Some(gpu) = system_resources.gpu_resources.first() {
            (gpu.available_vram_bytes, format!("GPU VRAM"))
        } else {
            return Ok(ValidationResult::Insufficient {
                message: format!("GPU not found but device is set to GPU"),
            });
        }
    } else {
        (system_resources.available_ram_bytes, "RAM".to_string())
    };

    let safe_threshold = (available as f64 * 0.8) as u64;
    let tight_threshold = (available as f64 * 0.95) as u64;

    if estimated_memory <= safe_threshold {
        Ok(ValidationResult::Sufficient)
    } else if estimated_memory <= tight_threshold {
        let message = format!(
            "Memory is tight: need {:.1} GB, available {:.1} GB {}",
            estimated_memory as f64 / 1_000_000_000.0,
            available as f64 / 1_000_000_000.0,
            resource_name
        );
        Ok(ValidationResult::Warning { message })
    } else {
        let message = format!(
            "Insufficient memory: need {:.1} GB, only {:.1} GB {} available",
            estimated_memory as f64 / 1_000_000_000.0,
            available as f64 / 1_000_000_000.0,
            resource_name
        );

        match mode {
            ResourceMode::Strict => Ok(ValidationResult::Insufficient { message }),
            ResourceMode::Loose => Ok(ValidationResult::Warning { message }),
        }
    }
}
