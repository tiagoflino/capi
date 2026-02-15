//! C++ bridge definitions for OpenVINO GenAI using cxx.
//!
//! This module defines the FFI boundary between Rust and the C++ OpenVINO GenAI library.

#[cxx::bridge(namespace = "genai_bridge")]
pub mod ffi {
    // Shared structs (passed by value between Rust and C++)
    #[derive(Debug, Clone, Default)]
    pub struct PerfMetricsData {
        pub load_time: f32,
        pub num_input_tokens: usize,
        pub num_generated_tokens: usize,
        pub ttft_mean: f32,
        pub ttft_std: f32,
        pub throughput_mean: f32,
        pub throughput_std: f32,
        pub generate_duration_mean: f32,
        pub generate_duration_std: f32,
    }

    #[derive(Debug)]
    pub struct GenerationResultData {
        pub text: String,
        pub metrics: PerfMetricsData,
    }

    extern "Rust" {
        type StreamerCallback<'a>;
        fn on_token(self: &mut StreamerCallback, token: &[u8]) -> bool;
    }

    unsafe extern "C++" {
        include!("capi-core/src/cpp/bridge.h");

        // Opaque types
        type LLMPipelineWrapper;
        type GenerationConfigWrapper;
        type TokenizerWrapper;

        // Factory functions
        fn create_pipeline(model_path: &str, device: &str) -> Result<UniquePtr<LLMPipelineWrapper>>;
        fn create_generation_config() -> Result<UniquePtr<GenerationConfigWrapper>>;

        // Tokenizer methods
        fn pipeline_get_tokenizer(pipeline: &LLMPipelineWrapper) -> UniquePtr<TokenizerWrapper>;
        fn tokenizer_count_tokens(tokenizer: Pin<&mut TokenizerWrapper>, text: &str) -> usize;

        // Pipeline methods
        fn pipeline_generate(
            pipeline: &LLMPipelineWrapper,
            prompt: &str,
            config: &GenerationConfigWrapper,
        ) -> String;

        fn pipeline_generate_with_metrics(
            pipeline: &LLMPipelineWrapper,
            prompt: &str,
            config: &GenerationConfigWrapper,
        ) -> GenerationResultData;

        // streaming now takes a Rust object reference
        fn pipeline_generate_stream(
            pipeline: &LLMPipelineWrapper,
            prompt: &str,
            config: &GenerationConfigWrapper,
            callback: &mut StreamerCallback,
        ) -> GenerationResultData;

        fn pipeline_start_chat(pipeline: Pin<&mut LLMPipelineWrapper>);
        fn pipeline_finish_chat(pipeline: Pin<&mut LLMPipelineWrapper>);

        // Config methods
        fn config_set_max_new_tokens(config: Pin<&mut GenerationConfigWrapper>, max_tokens: usize);
        fn config_set_temperature(config: Pin<&mut GenerationConfigWrapper>, temperature: f32);
        fn config_set_top_p(config: Pin<&mut GenerationConfigWrapper>, top_p: f32);
        fn config_set_top_k(config: Pin<&mut GenerationConfigWrapper>, top_k: usize);
        fn config_set_do_sample(config: Pin<&mut GenerationConfigWrapper>, do_sample: bool);
        fn config_set_stop_strings(config: Pin<&mut GenerationConfigWrapper>, stop_strings: Vec<String>);
        fn config_set_frequency_penalty(config: Pin<&mut GenerationConfigWrapper>, frequency_penalty: f32);
        fn config_set_presence_penalty(config: Pin<&mut GenerationConfigWrapper>, presence_penalty: f32);
        fn config_set_repetition_penalty(config: Pin<&mut GenerationConfigWrapper>, repetition_penalty: f32);
        fn config_set_rng_seed(config: Pin<&mut GenerationConfigWrapper>, seed: usize);
        fn config_set_logprobs(config: Pin<&mut GenerationConfigWrapper>, logprobs: usize);
    }
}

// The Rust struct that holds the closure and a buffer for partial UTF-8 sequences
pub struct StreamerCallback<'a> {
    pub cb: Box<dyn FnMut(&str) -> bool + 'a>,
    pub buffer: Vec<u8>,
}

impl<'a> StreamerCallback<'a> {
    pub fn on_token(&mut self, token: &[u8]) -> bool {
        self.buffer.extend_from_slice(token);
        
        // Try to decode the buffer as UTF-8
        match String::from_utf8(self.buffer.clone()) {
            Ok(s) => {
                // Entire buffer is valid UTF-8
                self.buffer.clear();
                (self.cb)(&s)
            }
            Err(e) => {
                // Buffer is not valid UTF-8, but might have a valid prefix
                let valid_up_to = e.utf8_error().valid_up_to();
                if valid_up_to > 0 {
                    let valid_bytes = self.buffer[..valid_up_to].to_vec();
                    // SAFETY: valid_up_to is guaranteed to be a valid UTF-8 boundary
                    let s = unsafe { String::from_utf8_unchecked(valid_bytes) };
                    
                    // Remove the valid prefix from the buffer
                    self.buffer.drain(0..valid_up_to);
                    
                    (self.cb)(&s)
                } else {
                    // No valid UTF-8 prefix found yet, keep waiting for more bytes
                    true
                }
            }
        }
    }
}
