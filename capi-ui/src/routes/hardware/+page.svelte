<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from 'svelte';

  interface DeviceInfo {
    name: string;
    device_type: string;
    available: boolean;
  }

  interface HardwareStatus {
    available_devices: DeviceInfo[];
    selected_device?: string;
  }

  let hardwareStatus = $state<HardwareStatus | null>(null);
  let loading = $state(true);

  onMount(async () => {
    await loadHardware();
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
</script>

<div style="height: 100%; overflow-y: auto;">
  <div style="padding: 32px; max-width: 800px; margin: 0 auto;">
    <h1 style="font-size: 32px; font-weight: bold; margin-bottom: 32px;">Hardware</h1>

    {#if loading}
      <div style="text-align: center; padding: 48px;">
        <div style="display: inline-block; width: 32px; height: 32px; border: 4px solid #282828; border-top-color: #3b82f6; border-radius: 50%; animation: spin 1s linear infinite;"></div>
        <p style="color: #888; margin-top: 16px;">Detecting hardware...</p>
      </div>
    {:else if hardwareStatus}
      <div style="display: flex; flex-direction: column; gap: 24px;">
        <!-- Available Devices -->
        <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 24px;">
          <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 16px;">Available Devices</h2>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            {#each hardwareStatus.available_devices as device}
              <div style="display: flex; align-items: center; gap: 12px;">
                <div style="width: 12px; height: 12px; border-radius: 50%; background: {device.available ? '#22c55e' : '#ef4444'};"></div>
                <span style="color: white; font-size: 15px; font-weight: 500;">{device.name}</span>
                <span style="color: #666; font-size: 13px;">({device.device_type})</span>
              </div>
            {/each}
          </div>
        </div>

        <!-- Selected Device -->
        {#if hardwareStatus.selected_device}
          <div style="background: linear-gradient(135deg, #1e3a8a, #5b21b6); border: 1px solid #3b82f6; border-radius: 12px; padding: 20px;">
            <p style="font-size: 14px; font-weight: 500; color: #bfdbfe;">
              Selected Device: <span style="font-weight: bold; color: white;">{hardwareStatus.selected_device}</span>
            </p>
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
