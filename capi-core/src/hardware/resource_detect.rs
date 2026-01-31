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

    let gpu_resources = detect_gpu_resources(&sys);

    Ok(SystemResources {
        total_ram_bytes,
        available_ram_bytes,
        gpu_resources,
    })
}

fn detect_gpu_resources(sys: &System) -> Vec<GpuResource> {
    let mut resources = Vec::new();

    let intel_gpus = detect_intel_gpu(sys);
    resources.extend(intel_gpus);

    resources
}

#[cfg(target_os = "linux")]
fn detect_intel_gpu(sys: &System) -> Vec<GpuResource> {
    // First try xe driver sysfs paths (newer Intel GPUs)
    let xe_gpus = detect_intel_gpu_xe(sys);
    if !xe_gpus.is_empty() {
        return xe_gpus;
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
                return vec![GpuResource {
                    name: "Intel GPU".to_string(),
                    total_vram_bytes: total_bytes,
                    available_vram_bytes: total_bytes.saturating_sub(used_bytes),
                    device_type: DeviceType::GPU,
                    usage_percent: 0.0,
                    frequency_mhz: 0,
                    max_frequency_mhz: 0,
                }];
            }
        }
    }

    detect_intel_gpu_via_clinfo().into_iter().collect()
}

/// Detect Intel GPU using xe driver sysfs paths (Meteor Lake, Lunar Lake, etc.)
#[cfg(target_os = "linux")]
fn detect_intel_gpu_xe(sys: &System) -> Vec<GpuResource> {
    let mut gpus = Vec::new();

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

        // Check for xe driver tile/gt structure or frequency path
        let freq_path = format!("/sys/class/drm/card{}/device/tile0/gt0/freq0", card);
        let gtidle_path = format!("/sys/class/drm/card{}/device/tile0/gt0/gtidle", card);

        if !std::path::Path::new(&freq_path).exists() {
            continue;
        }

        // Read frequency metrics (xe uses cur_freq, not act_freq)
        let act_freq = std::fs::read_to_string(format!("{}/cur_freq", freq_path))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);
        
        let max_freq = std::fs::read_to_string(format!("{}/rp0_freq", freq_path))
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

        // Try to read explicit VRAM info (Discrete GPU)
        let total_vram_path = format!("/sys/class/drm/card{}/device/mem_info_vram_total", card);
        let used_vram_path = format!("/sys/class/drm/card{}/device/mem_info_vram_used", card);

        let mut gpu_total = 0;
        let mut gpu_available = 0;
        let mut found_vram = false;

        if let (Ok(t_str), Ok(u_str)) = (std::fs::read_to_string(&total_vram_path), std::fs::read_to_string(&used_vram_path)) {
            if let (Ok(t), Ok(u)) = (t_str.trim().parse::<u64>(), u_str.trim().parse::<u64>()) {
                gpu_total = t;
                gpu_available = t.saturating_sub(u);
                found_vram = true;
            }
        }

        if !found_vram {
            // For integrated GPUs, use system RAM as shared memory
            // Re-read fresh memory values
            let mut fresh_sys = System::new_all();
            fresh_sys.refresh_memory();
            
            let total_ram = fresh_sys.total_memory();
            let available_ram = fresh_sys.available_memory();

            // Shared memory is typically up to 50% of system RAM
            // Both total and available are halved to track shared VRAM proportionally
            gpu_total = total_ram / 2;
            gpu_available = available_ram / 2;
        }

        gpus.push(GpuResource {
            name: gpu_name,
            total_vram_bytes: gpu_total,
            available_vram_bytes: gpu_available,
            device_type: DeviceType::GPU,
            usage_percent,
            frequency_mhz: act_freq,
            max_frequency_mhz: max_freq,
        });
    }

    gpus
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
