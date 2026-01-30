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

<div class="hardware-page">
  <div class="content-container">
    <header class="section-header">
      <h1>Hardware Architecture</h1>
      <p>Real-time telemetry and device monitoring</p>
    </header>

    {#if loading}
      <div class="loading-container">
        <div class="spinner"></div>
        <p>Probing hardware interfaces...</p>
      </div>
    {:else}
      <div class="hardware-grid">
        <!-- GPU Elevation -->
        {#if resources && resources.gpu_resources.length > 0}
          {#each resources.gpu_resources as gpu}
            <div class="card gpu-card">
              <div class="card-header">
                <div>
                  <h2 class="card-title">{gpu.name}</h2>
                  <span class="card-subtitle">Integrated Graphics</span>
                </div>
                <div class="badge {gpu.usage_percent > 0 ? 'active' : ''}">
                  {gpu.usage_percent > 0 ? 'ACTIVE' : 'IDLE'}
                </div>
              </div>
              
              <div class="metric-group">
                <div class="metric-meta">
                  <span>Utilization</span>
                  <span class="value">{gpu.usage_percent.toFixed(1)}%</span>
                </div>
                <div class="progress-track">
                  <div class="progress-fill accent" style="width: {gpu.usage_percent}%;"></div>
                </div>
              </div>

              <div class="metric-group">
                <div class="metric-meta">
                  <span>Engine Clock</span>
                  <span class="value">{gpu.frequency_mhz} MHz</span>
                </div>
                <div class="progress-track">
                  <div class="progress-fill green" style="width: {gpu.max_frequency_mhz > 0 ? (gpu.frequency_mhz / gpu.max_frequency_mhz * 100) : 0}%;"></div>
                </div>
              </div>

              <div class="metric-group">
                <div class="metric-meta">
                  <span>VRAM Allocation</span>
                  <span class="value">{formatBytes(gpu.total_vram_bytes - gpu.available_vram_bytes)} / {formatBytes(gpu.total_vram_bytes)}</span>
                </div>
                <div class="progress-track">
                  <div class="progress-fill red" style="width: {((gpu.total_vram_bytes - gpu.available_vram_bytes) / gpu.total_vram_bytes * 100)}%;"></div>
                </div>
              </div>
            </div>
          {/each}
        {/if}

        <div class="stats-row">
          {#if resources}
            <div class="card stat-card">
              <span class="stat-label">CPU LOAD</span>
              <div class="stat-main">
                <span class="stat-value">{resources.cpu_usage_percent.toFixed(1)}</span>
                <span class="stat-unit">%</span>
              </div>
              <div class="mini-progress">
                <div class="mini-fill" style="width: {resources.cpu_usage_percent}%;"></div>
              </div>
            </div>

            <div class="card stat-card">
              <span class="stat-label">MEMORY PRESSURE</span>
              <div class="stat-main">
                <span class="stat-value">{formatBytes(resources.total_ram_bytes - resources.available_ram_bytes).split(' ')[0]}</span>
                <span class="stat-unit">{formatBytes(resources.total_ram_bytes - resources.available_ram_bytes).split(' ')[1]}</span>
              </div>
              <div class="mini-progress">
                <div class="mini-fill" style="width: {((resources.total_ram_bytes - resources.available_ram_bytes) / resources.total_ram_bytes * 100)}%;"></div>
              </div>
              <p class="stat-subtext">{formatBytes(resources.available_ram_bytes)} free of {formatBytes(resources.total_ram_bytes)}</p>
            </div>
          {/if}
        </div>

        <!-- Available Devices -->
        {#if hardwareStatus}
          <div class="card devices-card">
            <h2 class="card-title-small">Connected Backend Devices</h2>
            <div class="device-list">
              {#each hardwareStatus.available_devices as device}
                <div class="device-item">
                  <div class="status-indicator {device.available ? 'available' : 'unavailable'}"></div>
                  <div class="device-info">
                    <span class="d-name">{device.name}</span>
                    <span class="d-type">{device.device_type}</span>
                  </div>
                  {#if resources?.selected_device === device.name}
                    <span class="in-use-pill">Primary</span>
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
  .hardware-page {
    height: 100%;
    overflow-y: auto;
    background: #fcfaf7;
    padding-bottom: 80px;
  }

  .content-container {
    max-width: 860px;
    margin: 0 auto;
    padding: 40px 24px;
  }

  .section-header {
    margin-bottom: 40px;
    animation: fadeInDown 0.6s ease-out;
  }

  .section-header h1 {
    font-size: 32px;
    font-weight: 800;
    letter-spacing: -0.03em;
    color: #3d3b38;
    margin-bottom: 8px;
  }

  .section-header p {
    color: #8c8984;
    font-size: 16px;
    font-weight: 500;
  }

  .card {
    background: white;
    border: 1px solid rgba(0, 0, 0, 0.05);
    border-radius: 20px;
    padding: 24px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.03);
    transition: transform 0.2s;
  }

  .gpu-card {
    margin-bottom: 24px;
    border-top: 4px solid #b87333;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;
  }

  .card-title { font-size: 20px; font-weight: 800; color: #3d3b38; }
  .card-subtitle { font-size: 12px; font-weight: 600; color: #8c8984; text-transform: uppercase; letter-spacing: 0.05em; }

  .badge {
    padding: 4px 10px;
    border-radius: 20px;
    font-size: 10px;
    font-weight: 700;
    background: #f2f2f7;
    color: #8c8984;
  }
  .badge.active { background: #dcfce7; color: #16a34a; }

  .metric-group { margin-bottom: 20px; }
  .metric-meta { display: flex; justify-content: space-between; font-size: 12px; font-weight: 700; color: #8c8984; margin-bottom: 6px; }
  .metric-meta .value { color: #3d3b38; }

  .progress-track { height: 8px; background: rgba(0, 0, 0, 0.05); border-radius: 4px; overflow: hidden; }
  .progress-fill { height: 100%; border-radius: 4px; transition: width 0.8s cubic-bezier(0.16, 1, 0.3, 1); }
  .progress-fill.accent { background: #b87333; }
  .progress-fill.green { background: #22c55e; }
  .progress-fill.red { background: #ef4444; }

  .stats-row { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 24px; }
  .stat-card { display: flex; flex-direction: column; }
  .stat-label { font-size: 11px; font-weight: 700; color: #8c8984; letter-spacing: 0.05em; margin-bottom: 12px; }
  .stat-main { display: flex; align-items: baseline; gap: 4px; margin-bottom: 12px; }
  .stat-value { font-size: 32px; font-weight: 800; color: #3d3b38; letter-spacing: -0.02em; }
  .stat-unit { font-size: 14px; font-weight: 600; color: #8c8984; }
  .stat-subtext { font-size: 11px; color: #8c8984; margin-top: 8px; }

  .mini-progress { height: 4px; background: rgba(0, 0, 0, 0.05); border-radius: 2px; overflow: hidden; }
  .mini-fill { height: 100%; background: #3d3b38; border-radius: 2px; }

  .card-title-small { font-size: 14px; font-weight: 700; color: #3d3b38; margin-bottom: 20px; }
  .device-list { display: flex; flex-direction: column; gap: 12px; }
  .device-item { display: flex; align-items: center; gap: 16px; padding: 12px; border-radius: 12px; background: #faf8f5; }
  .status-indicator { width: 8px; height: 8px; border-radius: 50%; }
  .status-indicator.available { background: #22c55e; box-shadow: 0 0 8px rgba(34, 197, 94, 0.4); }
  .status-indicator.unavailable { background: #ef4444; }
  .device-info { flex: 1; display: flex; flex-direction: column; }
  .d-name { font-size: 14px; font-weight: 600; color: #3d3b38; }
  .d-type { font-size: 11px; color: #8c8984; }
  .in-use-pill { background: #fae8d1; color: #b87333; font-size: 10px; font-weight: 800; padding: 4px 10px; border-radius: 8px; text-transform: uppercase; }

  .loading-container { text-align: center; padding: 80px 0; color: #8c8984; }
  .spinner { width: 32px; height: 32px; border: 3px solid rgba(184, 115, 51, 0.1); border-top-color: #b87333; border-radius: 50%; margin: 0 auto 20px; animation: spin 1s linear infinite; }

  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes fadeInDown { from { opacity: 0; transform: translateY(-10px); } to { opacity: 1; transform: translateY(0); } }
</style>
