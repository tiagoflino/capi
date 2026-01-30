//! LLM Pipeline wrapper for OpenVINO GenAI.

use super::{GenAIError, Result, GenerationConfig, PerfMetrics};
use crate::genai_bridge::{ffi, StreamerCallback};
use cxx::UniquePtr;

/// Result of text generation including output text and performance metrics.
pub struct GenerationResult {
    pub text: String,
    pub metrics: PerfMetrics,
}

/// Wrapper around OpenVINO GenAI LLMPipeline for text generation.
pub struct LLMPipeline {
    inner: UniquePtr<ffi::LLMPipelineWrapper>,
}

unsafe impl Send for LLMPipeline {}
unsafe impl Sync for LLMPipeline {}

impl LLMPipeline {
    /// Create a new LLMPipeline from a model path and device.
    ///
    /// # Arguments
    /// * `model_path` - Path to the model directory
    /// * `device` - Device to use (e.g., "CPU", "GPU", "NPU")
    pub fn new(model_path: &str, device: &str) -> Result<Self> {
        let inner = ffi::create_pipeline(model_path, device);
        
        if inner.is_null() {
            return Err(GenAIError::PipelineCreation);
        }

        Ok(Self { inner })
    }

    /// Generate text from a prompt.
    ///
    /// # Arguments
    /// * `prompt` - The input prompt
    /// * `max_tokens` - Maximum number of tokens to generate
    pub fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        let mut config = GenerationConfig::new()?;
        config.set_max_new_tokens(max_tokens)?;
        
        let text = ffi::pipeline_generate(&self.inner, prompt, config.inner());
        Ok(text)
    }

    /// Generate text with full configuration control.
    pub fn generate_with_config(&self, prompt: &str, config: &GenerationConfig) -> Result<String> {
        let text = ffi::pipeline_generate(&self.inner, prompt, config.inner());
        Ok(text)
    }

    /// Generate text and return performance metrics.
    pub fn generate_with_metrics(&self, prompt: &str, max_tokens: usize) -> Result<GenerationResult> {
        let mut config = GenerationConfig::new()?;
        config.set_max_new_tokens(max_tokens)?;
        
        let result = ffi::pipeline_generate_with_metrics(&self.inner, prompt, config.inner());
        
        Ok(GenerationResult {
            text: result.text,
            metrics: PerfMetrics::from_data(result.metrics),
        })
    }

    /// Generate text with streaming, calling the callback for each token.
    ///
    /// # Arguments
    /// * `prompt` - The input prompt
    /// * `max_tokens` - Maximum number of tokens to generate
    /// * `callback` - Function called with each generated token, return false to stop
    pub fn generate_stream<F>(
        &self,
        prompt: &str,
        max_tokens: usize,
        callback: F,
    ) -> Result<GenerationResult>
    where
        F: FnMut(&str) -> bool,
    {
        let mut config = GenerationConfig::new()?;
        config.set_max_new_tokens(max_tokens)?;
        
        let mut streamer = StreamerCallback {
            cb: Box::new(callback),
            buffer: Vec::new(),
        };

        let result = ffi::pipeline_generate_stream(&self.inner, prompt, config.inner(), &mut streamer);
        
        Ok(GenerationResult {
            text: result.text,
            metrics: PerfMetrics::from_data(result.metrics),
        })
    }

    /// Generate text with streaming using full config.
    pub fn generate_stream_with_config<F>(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        callback: F,
    ) -> Result<GenerationResult>
    where
        F: FnMut(&str) -> bool,
    {
        let mut streamer = StreamerCallback {
            cb: Box::new(callback),
            buffer: Vec::new(),
        };

        let result = ffi::pipeline_generate_stream(&self.inner, prompt, config.inner(), &mut streamer);
        
        Ok(GenerationResult {
            text: result.text,
            metrics: PerfMetrics::from_data(result.metrics),
        })
    }

    /// Start a chat session (maintains KV cache between generations).
    pub fn start_chat(&mut self) -> Result<()> {
        ffi::pipeline_start_chat(self.inner.pin_mut());
        Ok(())
    }

    /// Finish a chat session and clear the KV cache.
    pub fn finish_chat(&mut self) -> Result<()> {
        ffi::pipeline_finish_chat(self.inner.pin_mut());
        Ok(())
    }

    /// Count tokens in a string using the pipeline's tokenizer.
    pub fn count_tokens(&self, text: &str) -> usize {
        let mut tokenizer = ffi::pipeline_get_tokenizer(&self.inner);
        if tokenizer.is_null() {
            return 0;
        }
        ffi::tokenizer_count_tokens(tokenizer.pin_mut(), text)
    }
}
