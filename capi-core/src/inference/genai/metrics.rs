use super::{ffi, check_status, Result};

pub struct PerfMetrics {
    raw: *mut ffi::ov_genai_perf_metrics,
}

unsafe impl Send for PerfMetrics {}
unsafe impl Sync for PerfMetrics {}

impl PerfMetrics {
    pub(crate) fn from_raw(raw: *mut ffi::ov_genai_perf_metrics) -> Self {
        Self { raw }
    }

    pub fn load_time(&self) -> Result<f32> {
        unsafe {
            let mut time = 0.0f32;
            check_status(ffi::ov_genai_perf_metrics_get_load_time(self.raw, &mut time))?;
            Ok(time)
        }
    }

    pub fn num_input_tokens(&self) -> Result<usize> {
        unsafe {
            let mut count = 0usize;
            check_status(ffi::ov_genai_perf_metrics_get_num_input_tokens(self.raw, &mut count))?;
            Ok(count)
        }
    }

    pub fn num_generated_tokens(&self) -> Result<usize> {
        unsafe {
            let mut count = 0usize;
            check_status(ffi::ov_genai_perf_metrics_get_num_generation_tokens(self.raw, &mut count))?;
            Ok(count)
        }
    }

    pub fn ttft(&self) -> Result<(f32, f32)> {
        unsafe {
            let mut mean = 0.0f32;
            let mut std = 0.0f32;
            check_status(ffi::ov_genai_perf_metrics_get_ttft(self.raw, &mut mean, &mut std))?;
            Ok((mean, std))
        }
    }

    pub fn throughput(&self) -> Result<(f32, f32)> {
        unsafe {
            let mut mean = 0.0f32;
            let mut std = 0.0f32;
            check_status(ffi::ov_genai_perf_metrics_get_throughput(self.raw, &mut mean, &mut std))?;
            Ok((mean, std))
        }
    }

    pub fn generate_duration(&self) -> Result<(f32, f32)> {
        unsafe {
            let mut mean = 0.0f32;
            let mut std = 0.0f32;
            check_status(ffi::ov_genai_perf_metrics_get_generate_duration(self.raw, &mut mean, &mut std))?;
            Ok((mean, std))
        }
    }
}

impl Drop for PerfMetrics {
    fn drop(&mut self) {
        unsafe {
            if !self.raw.is_null() {
                ffi::ov_genai_decoded_results_perf_metrics_free(self.raw);
            }
        }
    }
}
