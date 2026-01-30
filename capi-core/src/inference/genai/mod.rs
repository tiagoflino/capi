mod ffi;
mod pipeline;
mod config;
mod metrics;

pub use pipeline::{LLMPipeline, GenerationResult};
pub use config::GenerationConfig;
pub use metrics::PerfMetrics;

use std::ffi::CStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenAIError {
    #[error("OpenVINO GenAI error: {0}")]
    General(String),

    #[error("Failed to create pipeline")]
    PipelineCreation,

    #[error("Generation failed: {0}")]
    Generation(String),

    #[error("Invalid UTF-8 in output")]
    InvalidUtf8,
}

pub type Result<T> = std::result::Result<T, GenAIError>;

fn check_status(status: ffi::ov_status_e) -> Result<()> {
    if status == ffi::ov_status_e_OK {
        Ok(())
    } else {
        let detail = unsafe {
            let msg_ptr = ffi::ov_get_last_err_msg();
            if !msg_ptr.is_null() {
                CStr::from_ptr(msg_ptr).to_string_lossy().into_owned()
            } else {
                "No additional details".to_string()
            }
        };
        let err_msg = format!("OpenVINO GenAI error status code: {} ({})", status as i32, detail);
        println!("{}", err_msg);
        Err(GenAIError::General(err_msg))
    }
}
