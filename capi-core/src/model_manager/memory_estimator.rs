use anyhow::Result;

#[derive(Debug, Clone)]
pub struct MemoryEstimate {
    pub file_size_bytes: u64,
    pub estimated_runtime_bytes: u64,
}

pub fn estimate_memory_from_file_size(file_size_bytes: u64) -> Result<MemoryEstimate> {
    // Ballpark: runtime memory is roughly 1.5x file size
    // This accounts for model weights + KV cache + overhead
    let estimated_runtime_bytes = (file_size_bytes as f64 * 1.5) as u64;

    Ok(MemoryEstimate {
        file_size_bytes,
        estimated_runtime_bytes,
    })
}
