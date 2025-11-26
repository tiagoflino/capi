mod ffi;
mod pipeline;
mod config;
mod metrics;

pub use pipeline::{LLMPipeline, GenerationResult};
pub use config::GenerationConfig;
pub use metrics::PerfMetrics;

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
        Err(GenAIError::General(format!("Status code {}", status as i32)))
    }
}
