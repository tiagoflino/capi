//! Performance metrics from OpenVINO GenAI.

use crate::genai_bridge::ffi::PerfMetricsData;

/// Performance metrics collected during text generation.
#[derive(Debug, Clone)]
pub struct PerfMetrics {
    data: PerfMetricsData,
}

impl PerfMetrics {
    /// Create PerfMetrics from FFI data.
    pub(crate) fn from_data(data: PerfMetricsData) -> Self {
        Self { data }
    }

    /// Get the model load time in milliseconds.
    pub fn load_time(&self) -> f32 {
        self.data.load_time
    }

    /// Get the number of tokens in the input prompt.
    pub fn num_input_tokens(&self) -> usize {
        self.data.num_input_tokens
    }

    /// Get the number of tokens generated.
    pub fn num_generated_tokens(&self) -> usize {
        self.data.num_generated_tokens
    }

    /// Get Time To First Token (TTFT) as (mean, std) in milliseconds.
    pub fn ttft(&self) -> (f32, f32) {
        (self.data.ttft_mean, self.data.ttft_std)
    }

    /// Get throughput as (mean, std) in tokens per second.
    pub fn throughput(&self) -> (f32, f32) {
        (self.data.throughput_mean, self.data.throughput_std)
    }

    /// Get generation duration as (mean, std) in milliseconds.
    pub fn generate_duration(&self) -> (f32, f32) {
        (self.data.generate_duration_mean, self.data.generate_duration_std)
    }
}

impl Default for PerfMetrics {
    fn default() -> Self {
        Self {
            data: PerfMetricsData::default(),
        }
    }
}
