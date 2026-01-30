//! OpenVINO GenAI inference module.
//!
//! This module provides Rust wrappers for the OpenVINO GenAI C++ library
//! using cxx for safe FFI.

mod pipeline;
mod config;
mod metrics;

pub use pipeline::{LLMPipeline, GenerationResult};
pub use config::GenerationConfig;
pub use metrics::PerfMetrics;

use thiserror::Error;

/// Errors that can occur during GenAI operations.
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

/// Result type for GenAI operations.
pub type Result<T> = std::result::Result<T, GenAIError>;
