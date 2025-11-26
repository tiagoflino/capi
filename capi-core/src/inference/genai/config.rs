use super::{ffi, check_status, Result};
use std::ptr;

pub struct GenerationConfig {
    raw: *mut ffi::ov_genai_generation_config,
}

unsafe impl Send for GenerationConfig {}
unsafe impl Sync for GenerationConfig {}

impl GenerationConfig {
    pub fn new() -> Result<Self> {
        unsafe {
            let mut raw = ptr::null_mut();
            let status = ffi::ov_genai_generation_config_create(&mut raw);
            check_status(status)?;

            Ok(Self { raw })
        }
    }

    pub fn set_max_new_tokens(&mut self, max_tokens: usize) -> Result<()> {
        unsafe {
            let status = ffi::ov_genai_generation_config_set_max_new_tokens(self.raw, max_tokens);
            check_status(status)
        }
    }

    pub(crate) fn raw(&self) -> *mut ffi::ov_genai_generation_config {
        self.raw
    }
}

impl Drop for GenerationConfig {
    fn drop(&mut self) {
        unsafe {
            if !self.raw.is_null() {
                ffi::ov_genai_generation_config_free(self.raw);
            }
        }
    }
}
