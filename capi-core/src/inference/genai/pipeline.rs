use super::{ffi, check_status, GenAIError, Result, GenerationConfig, PerfMetrics};
use std::ffi::{CString, CStr};
use std::ptr;

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
