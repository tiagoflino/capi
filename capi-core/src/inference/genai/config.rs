//! Generation configuration for OpenVINO GenAI.

use super::Result;
use crate::genai_bridge::ffi;
use cxx::UniquePtr;

/// Configuration for text generation parameters.
pub struct GenerationConfig {
    inner: UniquePtr<ffi::GenerationConfigWrapper>,
}

// SAFETY: GenerationConfig wraps a UniquePtr to a C++ GenerationConfigWrapper.
// The wrapper only contains POD configuration values (max_tokens, temperature, top_p, etc.)
// and does not perform any operations that would cause data races. The config is copied
// into the pipeline before generation, so concurrent reads are safe.
unsafe impl Send for GenerationConfig {}
unsafe impl Sync for GenerationConfig {}

impl GenerationConfig {
    /// Create a new GenerationConfig with default settings.
    pub fn new() -> Result<Self> {
        let inner = ffi::create_generation_config();
        Ok(Self { inner })
    }

    /// Set the maximum number of new tokens to generate.
    pub fn set_max_new_tokens(&mut self, max_tokens: usize) -> Result<()> {
        ffi::config_set_max_new_tokens(self.inner.pin_mut(), max_tokens);
        Ok(())
    }

    /// Set the temperature for sampling (higher = more random).
    pub fn set_temperature(&mut self, temperature: f32) -> Result<()> {
        ffi::config_set_temperature(self.inner.pin_mut(), temperature);
        Ok(())
    }

    /// Set top-p (nucleus) sampling threshold.
    pub fn set_top_p(&mut self, top_p: f32) -> Result<()> {
        ffi::config_set_top_p(self.inner.pin_mut(), top_p);
        Ok(())
    }

    /// Set top-k sampling (number of top tokens to consider).
    pub fn set_top_k(&mut self, top_k: usize) -> Result<()> {
        ffi::config_set_top_k(self.inner.pin_mut(), top_k);
        Ok(())
    }

    /// Enable or disable sampling (vs greedy decoding).
    pub fn set_do_sample(&mut self, do_sample: bool) -> Result<()> {
        ffi::config_set_do_sample(self.inner.pin_mut(), do_sample);
        Ok(())
    }

    /// Set strings that will stop generation when encountered.
    pub fn set_stop_strings(&mut self, stop_strings: &[&str]) -> Result<()> {
        let strings: Vec<String> = stop_strings.iter().map(|s| s.to_string()).collect();
        ffi::config_set_stop_strings(self.inner.pin_mut(), strings);
        Ok(())
    }

    /// Get a reference to the inner wrapper for FFI calls.
    pub(crate) fn inner(&self) -> &ffi::GenerationConfigWrapper {
        &self.inner
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self::new().expect("Failed to create default GenerationConfig")
    }
}
