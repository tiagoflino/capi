use super::{ffi, check_status, GenAIError, Result, GenerationConfig, PerfMetrics};
use std::ffi::{CString, CStr};
use std::ptr;
use std::os::raw::c_char;

pub struct GenerationResult {
    pub text: String,
    pub metrics: PerfMetrics,
}

pub struct LLMPipeline {
    raw: *mut ffi::ov_genai_llm_pipeline,
}

unsafe impl Send for LLMPipeline {}
unsafe impl Sync for LLMPipeline {}

impl LLMPipeline {
    pub fn new(model_path: &str, device: &str) -> Result<Self> {
        unsafe {
            let path_c = CString::new(model_path)
                .map_err(|_| GenAIError::General("Invalid model path".to_string()))?;
            let device_c = CString::new(device)
                .map_err(|_| GenAIError::General("Invalid device string".to_string()))?;

            let mut raw = ptr::null_mut();
            let status = ffi::ov_genai_llm_pipeline_create(
                path_c.as_ptr(),
                device_c.as_ptr(),
                0,
                &mut raw,
            );

            check_status(status)?;

            if raw.is_null() {
                return Err(GenAIError::PipelineCreation);
            }

            Ok(Self { raw })
        }
    }

    pub fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        unsafe {
            let prompt_c = CString::new(prompt)
                .map_err(|_| GenAIError::General("Invalid prompt".to_string()))?;

            let mut config = GenerationConfig::new()?;
            config.set_max_new_tokens(max_tokens)?;

            let mut results = ptr::null_mut();
            let status = ffi::ov_genai_llm_pipeline_generate(
                self.raw,
                prompt_c.as_ptr(),
                config.raw(),
                ptr::null(),
                &mut results,
            );

            check_status(status)?;

            let mut output_size = 0;
            ffi::ov_genai_decoded_results_get_string(results, ptr::null_mut(), &mut output_size);

            let mut output_buf = vec![0u8; output_size];
            ffi::ov_genai_decoded_results_get_string(
                results,
                output_buf.as_mut_ptr() as *mut i8,
                &mut output_size,
            );

            ffi::ov_genai_decoded_results_free(results);

            if output_size > 0 && output_buf[output_size - 1] == 0 {
                output_buf.truncate(output_size - 1);
            }

            String::from_utf8(output_buf).map_err(|_| GenAIError::InvalidUtf8)
        }
    }

    pub fn start_chat(&self) -> Result<()> {
        unsafe {
            let status = ffi::ov_genai_llm_pipeline_start_chat(self.raw);
            check_status(status)
        }
    }

    pub fn finish_chat(&self) -> Result<()> {
        unsafe {
            let status = ffi::ov_genai_llm_pipeline_finish_chat(self.raw);
            check_status(status)
        }
    }

    pub fn generate_with_metrics(&self, prompt: &str, max_tokens: usize) -> Result<GenerationResult> {
        unsafe {
            let prompt_c = CString::new(prompt)
                .map_err(|_| GenAIError::General("Invalid prompt".to_string()))?;

            let mut config = GenerationConfig::new()?;
            config.set_max_new_tokens(max_tokens)?;
            config.set_stop_strings(&["<|im_end|>", "<|endoftext|>", "</s>", "<|im_start|>"])?;

            let mut results = ptr::null_mut();
            let status = ffi::ov_genai_llm_pipeline_generate(
                self.raw,
                prompt_c.as_ptr(),
                config.raw(),
                ptr::null(),
                &mut results,
            );

            check_status(status)?;

            let mut output_size = 0;
            ffi::ov_genai_decoded_results_get_string(results, ptr::null_mut(), &mut output_size);

            let mut output_buf = vec![0u8; output_size];
            ffi::ov_genai_decoded_results_get_string(
                results,
                output_buf.as_mut_ptr() as *mut i8,
                &mut output_size,
            );

            if output_size > 0 && output_buf[output_size - 1] == 0 {
                output_buf.truncate(output_size - 1);
            }

            let text = String::from_utf8(output_buf).map_err(|_| GenAIError::InvalidUtf8)?;

            let mut metrics_raw = ptr::null_mut();
            ffi::ov_genai_decoded_results_get_perf_metrics(results, &mut metrics_raw);
            let metrics = PerfMetrics::from_raw(metrics_raw);

            ffi::ov_genai_decoded_results_free(results);

            Ok(GenerationResult { text, metrics })
        }
    }

    pub fn generate_stream<F>(&self, prompt: &str, max_tokens: usize, mut callback: F) -> Result<GenerationResult>
    where
        F: FnMut(&str) -> bool,
    {
        unsafe {
            let prompt_c = CString::new(prompt)
                .map_err(|_| GenAIError::General("Invalid prompt".to_string()))?;

            let mut config = GenerationConfig::new()?;
            config.set_max_new_tokens(max_tokens)?;
            config.set_stop_strings(&["<|im_end|>", "<|endoftext|>", "</s>", "<|im_start|>"])?;

            let callback_ptr = &mut callback as *mut _ as *mut std::os::raw::c_void;

            extern "C" fn stream_callback<F>(
                token: *const c_char,
                user_data: *mut std::os::raw::c_void,
            ) -> ffi::ov_genai_streamming_status_e
            where
                F: FnMut(&str) -> bool,
            {
                unsafe {
                    if token.is_null() {
                        return ffi::ov_genai_streamming_status_e_OV_GENAI_STREAMMING_STATUS_RUNNING;
                    }

                    let callback = &mut *(user_data as *mut F);
                    let token_str = CStr::from_ptr(token).to_str().unwrap_or("");

                    if callback(token_str) {
                        ffi::ov_genai_streamming_status_e_OV_GENAI_STREAMMING_STATUS_RUNNING
                    } else {
                        ffi::ov_genai_streamming_status_e_OV_GENAI_STREAMMING_STATUS_CANCEL
                    }
                }
            }

            let streamer = ffi::streamer_callback {
                callback_func: Some(stream_callback::<F>),
                args: callback_ptr,
            };

            let mut results = ptr::null_mut();
            let status = ffi::ov_genai_llm_pipeline_generate(
                self.raw,
                prompt_c.as_ptr(),
                config.raw(),
                &streamer,
                &mut results,
            );

            check_status(status)?;

            let mut output_size = 0;
            ffi::ov_genai_decoded_results_get_string(results, ptr::null_mut(), &mut output_size);

            let mut output_buf = vec![0u8; output_size];
            ffi::ov_genai_decoded_results_get_string(
                results,
                output_buf.as_mut_ptr() as *mut i8,
                &mut output_size,
            );

            if output_size > 0 && output_buf[output_size - 1] == 0 {
                output_buf.truncate(output_size - 1);
            }

            let text = String::from_utf8(output_buf).map_err(|_| GenAIError::InvalidUtf8)?;

            let mut metrics_raw = ptr::null_mut();
            ffi::ov_genai_decoded_results_get_perf_metrics(results, &mut metrics_raw);
            let metrics = PerfMetrics::from_raw(metrics_raw);

            ffi::ov_genai_decoded_results_free(results);

            Ok(GenerationResult { text, metrics })
        }
    }
}

impl Drop for LLMPipeline {
    fn drop(&mut self) {
        unsafe {
            if !self.raw.is_null() {
                ffi::ov_genai_llm_pipeline_free(self.raw);
            }
        }
    }
}
