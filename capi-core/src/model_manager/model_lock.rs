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
            let pid_str = fs::read_to_string(&lock_path).unwrap_or_default();
            
            // Check if the process that holds the lock is still alive
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if Self::is_process_alive(pid) {
                    return Err(anyhow::anyhow!(
                        "Model '{}' is in use by another process (PID: {})",
                        model_id,
                        pid
                    ));
                } else {
                    // Process is dead, clean up stale lock
                    println!("Cleaning up stale lock for model '{}' (PID {} no longer exists)", model_id, pid);
                    fs::remove_file(&lock_path).ok();
                }
            } else {
                // Invalid PID in lock file, remove it
                println!("Removing invalid lock file for model '{}'", model_id);
                fs::remove_file(&lock_path).ok();
            }
        }

        // Create parent directory if needed
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        fs::write(&lock_path, std::process::id().to_string())?;

        Ok(Self { lock_path })
    }

    /// Force release a lock for a model (for cleanup purposes)
    pub fn force_release(model_id: &str) -> Result<()> {
        let lock_path = Self::lock_path(model_id);
        if lock_path.exists() {
            fs::remove_file(&lock_path)?;
            println!("Force released lock for model '{}'", model_id);
        }
        Ok(())
    }

    /// Clean up all stale locks from dead processes
    pub fn cleanup_stale_locks() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("capi-model-") && name.ends_with(".lock") {
                        if let Ok(pid_str) = fs::read_to_string(&path) {
                            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                if !Self::is_process_alive(pid) {
                                    println!("Cleaning stale lock: {:?} (PID {} dead)", path, pid);
                                    fs::remove_file(&path).ok();
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn lock_path(model_id: &str) -> PathBuf {
        let safe_id = model_id.replace(['/', '\\', ':'], "_");
        std::env::temp_dir().join(format!("capi-model-{}.lock", safe_id))
    }

    #[cfg(target_os = "linux")]
    fn is_process_alive(pid: u32) -> bool {
        // Check if /proc/{pid} exists on Linux
        std::path::Path::new(&format!("/proc/{}", pid)).exists()
    }

    #[cfg(target_os = "windows")]
    fn is_process_alive(pid: u32) -> bool {
        use std::process::Command;
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn is_process_alive(pid: u32) -> bool {
        // Fallback: assume alive (safer)
        true
    }
}

impl Drop for ModelLock {
    fn drop(&mut self) {
        fs::remove_file(&self.lock_path).ok();
    }
}
