use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct ModelLock {
    lock_path: PathBuf,
}

impl ModelLock {
    pub fn try_acquire(model_id: &str) -> Result<Self> {
        let lock_path = Self::lock_path(model_id);

        if lock_path.exists() {
            let pid = fs::read_to_string(&lock_path).unwrap_or_else(|_| "unknown".to_string());
            return Err(anyhow::anyhow!(
                "Model '{}' is in use by another process (PID: {})",
                model_id,
                pid
            ));
        }

        fs::write(&lock_path, std::process::id().to_string())?;

        Ok(Self { lock_path })
    }

    fn lock_path(model_id: &str) -> PathBuf {
        let safe_id = model_id.replace(['/', '\\', ':'], "_");
        std::env::temp_dir().join(format!("capi-model-{}.lock", safe_id))
    }
}

impl Drop for ModelLock {
    fn drop(&mut self) {
        fs::remove_file(&self.lock_path).ok();
    }
}
