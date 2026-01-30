<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from 'svelte';

  interface DeviceInfo {
    name: string;
    device_type: string;
    available: boolean;
  }

  interface GpuResource {
    name: string;
    total_vram_bytes: number;
    available_vram_bytes: number;
    usage_percent: number;
    frequency_mhz: number;
    max_frequency_mhz: number;
  }

  interface HardwareStatus {
    available_devices: DeviceInfo[];
    selected_device?: string;
  }

  interface SystemResources {
    total_ram_bytes: number;
    available_ram_bytes: number;
    cpu_usage_percent: number;
    selected_device?: string;
    gpu_resources: GpuResource[];
  }

  let hardwareStatus = $state<HardwareStatus | null>(null);
  let resources = $state<SystemResources | null>(null);
  let loading = $state(true);
  let refreshInterval: number;

  onMount(async () => {
    await loadHardware();
    await loadResources();
    
    // Refresh resources every 2 seconds
    refreshInterval = setInterval(async () => {
      await loadResources();
    }, 2000);

    return () => {
      if (refreshInterval) clearInterval(refreshInterval);
    };
  });

  async function loadHardware() {
    loading = true;
    try {
      hardwareStatus = await invoke('get_hardware_status');
    } catch (e) {
      console.error('Failed to load hardware:', e);
    }
    loading = false;
  }

  async function loadResources() {
    try {
      resources = await invoke('get_system_resources');
    } catch (e) {
      console.error('Failed to load resources:', e);
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes >= 1024 * 1024 * 1024) {
      return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
    }
    return (bytes / (1024 * 1024)).toFixed(0) + ' MB';
  }
</script>

<div style="height: 100%; overflow-y: auto;">
  <div style="padding: 32px; max-width: 900px; margin: 0 auto;">
    <h1 style="font-size: 32px; font-weight: bold; margin-bottom: 32px;">Hardware</h1>

    {#if loading}
      <div style="text-align: center; padding: 48px;">
        <div style="display: inline-block; width: 32px; height: 32px; border: 4px solid #282828; border-top-color: #3b82f6; border-radius: 50%; animation: spin 1s linear infinite;"></div>
        <p style="color: #888; margin-top: 16px;">Detecting hardware...</p>
      </div>
    {:else}
      <div style="display: flex; flex-direction: column; gap: 24px;">
        <!-- GPU Status Card -->
        {#if resources && resources.gpu_resources.length > 0}
          {#each resources.gpu_resources as gpu}
            <div style="background: linear-gradient(135deg, #0f172a, #1e1b4b); border: 1px solid #3b82f6; border-radius: 16px; padding: 24px;">
              <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <h2 style="font-size: 20px; font-weight: 600; color: white;">{gpu.name}</h2>
                <span style="background: #22c55e; color: white; padding: 4px 12px; border-radius: 12px; font-size: 12px; font-weight: 600;">ACTIVE</span>
              </div>
              
              <!-- GPU Usage -->
              <div style="margin-bottom: 20px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                  <span style="color: #94a3b8; font-size: 14px;">GPU Usage</span>
                  <span style="color: white; font-weight: 600; font-size: 14px;">{gpu.usage_percent.toFixed(1)}%</span>
                </div>
                <div style="background: #1e293b; border-radius: 8px; height: 12px; overflow: hidden;">
                  <div style="background: linear-gradient(90deg, #3b82f6, #8b5cf6); height: 100%; width: {gpu.usage_percent}%; transition: width 0.5s ease;"></div>
                </div>
              </div>

              <!-- GPU Frequency -->
              <div style="margin-bottom: 20px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                  <span style="color: #94a3b8; font-size: 14px;">Frequency</span>
                  <span style="color: white; font-weight: 600; font-size: 14px;">{gpu.frequency_mhz} / {gpu.max_frequency_mhz} MHz</span>
                </div>
                <div style="background: #1e293b; border-radius: 8px; height: 12px; overflow: hidden;">
                  <div style="background: linear-gradient(90deg, #10b981, #06b6d4); height: 100%; width: {gpu.max_frequency_mhz > 0 ? (gpu.frequency_mhz / gpu.max_frequency_mhz * 100) : 0}%; transition: width 0.5s ease;"></div>
                </div>
              </div>

              <!-- Memory -->
              <div>
                <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                  <span style="color: #94a3b8; font-size: 14px;">Shared Memory</span>
                  <span style="color: white; font-weight: 600; font-size: 14px;">
                    {formatBytes(gpu.total_vram_bytes - gpu.available_vram_bytes)} / {formatBytes(gpu.total_vram_bytes)}
                  </span>
                </div>
                <div style="background: #1e293b; border-radius: 8px; height: 12px; overflow: hidden;">
                  <div style="background: linear-gradient(90deg, #f59e0b, #ef4444); height: 100%; width: {((gpu.total_vram_bytes - gpu.available_vram_bytes) / gpu.total_vram_bytes * 100)}%; transition: width 0.5s ease;"></div>
                </div>
              </div>
            </div>
          {/each}
        {:else}
          <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 24px; text-align: center;">
            <p style="color: #888;">No GPU detected</p>
          </div>
        {/if}

        <!-- CPU & RAM Status -->
        {#if resources}
          <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
            <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 20px;">
              <h3 style="font-size: 14px; color: #888; margin-bottom: 12px;">CPU Usage</h3>
              <div style="font-size: 28px; font-weight: bold; color: white;">{resources.cpu_usage_percent.toFixed(1)}%</div>
              <div style="background: #282828; border-radius: 4px; height: 6px; margin-top: 12px; overflow: hidden;">
                <div style="background: #3b82f6; height: 100%; width: {resources.cpu_usage_percent}%;"></div>
              </div>
            </div>
            <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 20px;">
              <h3 style="font-size: 14px; color: #888; margin-bottom: 12px;">RAM Usage</h3>
              <div style="font-size: 28px; font-weight: bold; color: white;">
                {formatBytes(resources.total_ram_bytes - resources.available_ram_bytes)}
              </div>
              <div style="background: #282828; border-radius: 4px; height: 6px; margin-top: 12px; overflow: hidden;">
                <div style="background: #8b5cf6; height: 100%; width: {((resources.total_ram_bytes - resources.available_ram_bytes) / resources.total_ram_bytes * 100)}%;"></div>
              </div>
              <p style="font-size: 12px; color: #666; margin-top: 8px;">
                {formatBytes(resources.available_ram_bytes)} available of {formatBytes(resources.total_ram_bytes)}
              </p>
            </div>
          </div>
        {/if}

        <!-- Available Devices -->
        {#if hardwareStatus}
          <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 24px;">
            <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 16px;">Available Devices</h2>
            <div style="display: flex; flex-direction: column; gap: 12px;">
              {#each hardwareStatus.available_devices as device}
                <div style="display: flex; align-items: center; gap: 12px;">
                  <div style="width: 12px; height: 12px; border-radius: 50%; background: {device.available ? '#22c55e' : '#ef4444'};"></div>
                  <span style="color: white; font-size: 15px; font-weight: 500;">{device.name}</span>
                  <span style="color: #666; font-size: 13px;">({device.device_type})</span>
                  {#if resources?.selected_device === device.name}
                    <span style="background: #3b82f6; color: white; padding: 2px 8px; border-radius: 6px; font-size: 11px; font-weight: 600;">IN USE</span>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
