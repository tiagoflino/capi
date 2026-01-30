use anyhow::Result;
use super::DeviceType;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct SystemResources {
    pub total_ram_bytes: u64,
    pub available_ram_bytes: u64,
    pub gpu_resources: Vec<GpuResource>,
}

#[derive(Debug, Clone)]
pub struct GpuResource {
    pub name: String,
    pub total_vram_bytes: u64,
    pub available_vram_bytes: u64,
    pub device_type: DeviceType,
    pub usage_percent: f32,
    pub frequency_mhz: u32,
    pub max_frequency_mhz: u32,
}

pub fn detect_system_resources() -> Result<SystemResources> {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total_ram_bytes = sys.total_memory();
    let available_ram_bytes = sys.available_memory();

    let gpu_resources = detect_gpu_resources();

    Ok(SystemResources {
        total_ram_bytes,
        available_ram_bytes,
        gpu_resources,
    })
}

fn detect_gpu_resources() -> Vec<GpuResource> {
    let mut resources = Vec::new();

    if let Some(intel) = detect_intel_gpu() {
        resources.push(intel);
    }

    resources
}

#[cfg(target_os = "linux")]
fn detect_intel_gpu() -> Option<GpuResource> {
    // First try xe driver sysfs paths (newer Intel GPUs)
    if let Some(gpu) = detect_intel_gpu_xe() {
        return Some(gpu);
    }

    // Then try i915 driver paths (older Intel GPUs)
    for card in 0..10 {
        let total_path = format!("/sys/class/drm/card{}/device/mem_info_vram_total", card);
        let used_path = format!("/sys/class/drm/card{}/device/mem_info_vram_used", card);

        if let (Ok(total_str), Ok(used_str)) = (
            std::fs::read_to_string(&total_path),
            std::fs::read_to_string(&used_path)
        ) {
            if let (Ok(total_bytes), Ok(used_bytes)) = (
                total_str.trim().parse::<u64>(),
                used_str.trim().parse::<u64>()
            ) {
                return Some(GpuResource {
                    name: "Intel GPU".to_string(),
                    total_vram_bytes: total_bytes,
                    available_vram_bytes: total_bytes.saturating_sub(used_bytes),
                    device_type: DeviceType::GPU,
                    usage_percent: 0.0,
                    frequency_mhz: 0,
                    max_frequency_mhz: 0,
                });
            }
        }
    }

    detect_intel_gpu_via_clinfo()
}

/// Detect Intel GPU using xe driver sysfs paths (Meteor Lake, Lunar Lake, etc.)
#[cfg(target_os = "linux")]
fn detect_intel_gpu_xe() -> Option<GpuResource> {

    for card in 0..10 {
        let vendor_path = format!("/sys/class/drm/card{}/device/vendor", card);
        
        // Check if this is an Intel GPU (vendor 0x8086)
        if let Ok(vendor) = std::fs::read_to_string(&vendor_path) {
            if !vendor.trim().contains("0x8086") {
                continue;
            }
        } else {
            continue;
        }

        // Check for xe driver tile/gt structure
        let freq_path = format!("/sys/class/drm/card{}/device/tile0/gt0/freq0", card);
        let gtidle_path = format!("/sys/class/drm/card{}/device/tile0/gt0/gtidle", card);

        if !std::path::Path::new(&freq_path).exists() {
            continue;
        }

        // Read frequency metrics
        let act_freq = std::fs::read_to_string(format!("{}/act_freq", freq_path))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);
        
        let max_freq = std::fs::read_to_string(format!("{}/max_freq", freq_path))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);

        // Calculate GPU usage from idle residency
        let usage_percent = calculate_gpu_usage_from_idle(&gtidle_path);

        // Get GPU name from device ID
        let device_path = format!("/sys/class/drm/card{}/device/device", card);
        let device_id = std::fs::read_to_string(&device_path)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        
        let gpu_name = get_intel_gpu_name(&device_id);

        // For integrated GPUs, use system RAM as shared memory
        let mut sys = System::new(); // Don't need new_all, just memory
        sys.refresh_memory();
        
        let mut available_vram = sys.available_memory();
        let total_vram = sys.total_memory();
        
        // Fallback: if sysinfo available_memory is 0 but total is not, 
        // it might be a parsing issue. Try free_memory.
        if available_vram == 0 && total_vram > 0 {
            available_vram = sys.free_memory();
        }
        
        // If still 0, use a conservative 25% of total as "available" 
        // to avoid blocking user completely if detection fails
        if available_vram == 0 && total_vram > 0 {
            available_vram = total_vram / 4;
        }

        // Shared memory is typically up to 50% of system RAM
        let gpu_total = total_vram / 2;
        let gpu_available = available_vram.min(gpu_total);

        return Some(GpuResource {
            name: gpu_name,
            total_vram_bytes: gpu_total,
            available_vram_bytes: gpu_available,
            device_type: DeviceType::GPU,
            usage_percent,
            frequency_mhz: act_freq,
            max_frequency_mhz: max_freq,
        });
    }

    None
}

/// Calculate GPU usage from idle residency delta
#[cfg(target_os = "linux")]
fn calculate_gpu_usage_from_idle(gtidle_path: &str) -> f32 {
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;

    let idle_residency_path = format!("{}/idle_residency_ms", gtidle_path);
    
    // Read initial idle residency
    let idle1 = std::fs::read_to_string(&idle_residency_path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());

    if idle1.is_none() {
        return 0.0;
    }

    let idle1 = idle1.unwrap();
    let start = Instant::now();
    
    // Short sleep to measure delta
    thread::sleep(Duration::from_millis(50));
    
    let elapsed_ms = start.elapsed().as_millis() as u64;

    // Read second idle residency
    let idle2 = std::fs::read_to_string(&idle_residency_path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(idle1);

    let idle_delta = idle2.saturating_sub(idle1);
    
    if elapsed_ms == 0 {
        return 0.0;
    }

    // Usage = (elapsed - idle) / elapsed * 100
    let usage = ((elapsed_ms.saturating_sub(idle_delta)) as f32 / elapsed_ms as f32) * 100.0;
    usage.min(100.0).max(0.0)
}

/// Get Intel GPU name from device ID
fn get_intel_gpu_name(device_id: &str) -> String {
    match device_id {
        "0x64a0" | "0x64A0" => "Intel Arc (Lunar Lake)".to_string(),
        "0x7d55" | "0x7D55" => "Intel Arc (Meteor Lake)".to_string(),
        "0x56a0" | "0x56A0" => "Intel Arc A770".to_string(),
        "0x56a1" | "0x56A1" => "Intel Arc A750".to_string(),
        "0x5690" | "0x5691" => "Intel Arc A380".to_string(),
        _ => format!("Intel GPU ({})", device_id),
    }
}

#[cfg(target_os = "windows")]
fn detect_intel_gpu() -> Option<GpuResource> {
    use std::process::Command;

    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-WmiObject Win32_VideoController | Where-Object {$_.Name -like '*Intel*'} | ForEach-Object { \"$($_.Name)|$($_.AdapterRAM)\" }"
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let output_str = String::from_utf8(output.stdout).ok()?;
        let line = output_str.lines().next()?;
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() >= 2 {
            let name = parts[0].trim().to_string();
            if let Ok(adapter_ram) = parts[1].trim().parse::<u64>() {
                return Some(GpuResource {
                    name,
                    total_vram_bytes: adapter_ram,
                    available_vram_bytes: adapter_ram,
                    device_type: DeviceType::GPU,
                    usage_percent: 0.0,
                    frequency_mhz: 0,
                    max_frequency_mhz: 0,
                });
            }
        }
    }

    None
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn detect_intel_gpu() -> Option<GpuResource> {
    None
}

#[cfg(target_os = "linux")]
fn detect_intel_gpu_via_clinfo() -> Option<GpuResource> {
    use std::process::Command;

    let output = Command::new("clinfo")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8(output.stdout).ok()?;
    let mut is_intel = false;
    let mut global_mem = None;

    for line in output_str.lines() {
        if line.contains("Vendor") && line.to_lowercase().contains("intel") {
            is_intel = true;
        }
        if is_intel && line.contains("Global memory size") {
            if let Some(num_str) = line.split_whitespace().rev().nth(1) {
                if let Ok(bytes) = num_str.parse::<u64>() {
                    global_mem = Some(bytes);
                    break;
                }
            }
        }
    }

    if let Some(vram) = global_mem {
        Some(GpuResource {
            name: "Intel GPU".to_string(),
            total_vram_bytes: vram,
            available_vram_bytes: vram,
            device_type: DeviceType::GPU,
            usage_percent: 0.0,
            frequency_mhz: 0,
            max_frequency_mhz: 0,
        })
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
fn detect_intel_gpu_via_clinfo() -> Option<GpuResource> {
    None
}
