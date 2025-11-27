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
                });
            }
        }
    }

    detect_intel_gpu_via_clinfo()
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
        })
    } else {
        None
    }
}
