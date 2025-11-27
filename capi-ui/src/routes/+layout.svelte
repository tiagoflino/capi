<script lang="ts">
  import { page } from '$app/stores';
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from 'svelte';
  import ResourceGauge from '$lib/ResourceGauge.svelte';
  import '../app.css';

  let { children } = $props();

  const isActive = (path: string) => $page.url.pathname === path;

  interface SystemResources {
    total_ram_bytes: number;
    available_ram_bytes: number;
    cpu_usage_percent: number;
    selected_device?: string;
    gpu_resources: Array<{
      name: string;
      total_vram_bytes: number;
      available_vram_bytes: number;
    }>;
  }

  let resources = $state<SystemResources | null>(null);

  onMount(() => {
    updateResources();
    setInterval(updateResources, 2000);
  });

  async function updateResources() {
    try {
      resources = await invoke('get_system_resources');
    } catch (e) {
      console.error('Failed to get resources:', e);
    }
  }
</script>

<div style="display: flex; height: 100vh; background: #121212; color: white;">
  <!-- Sidebar -->
  <aside style="width: 240px; background: #000; display: flex; flex-direction: column;">
    <!-- Logo -->
    <div style="padding: 24px; border-bottom: 1px solid #282828;">
      <div style="display: flex; align-items: center; gap: 12px;">
        <img src="/capi-logo.png" alt="Capi" style="width: 32px; height: 32px; border-radius: 8px;" />
        <div>
          <h1 style="font-size: 18px; font-weight: bold;">Capi</h1>
          <p style="font-size: 12px; color: #888;">Local AI</p>
        </div>
      </div>
    </div>

    <!-- Navigation -->
    <nav style="flex: 1; padding: 12px;">
      <a
        href="/"
        style="display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 8px; text-decoration: none; margin-bottom: 4px; background: {isActive('/') ? '#282828' : 'transparent'}; color: {isActive('/') ? 'white' : '#888'};"
      >
        <svg style="width: 20px; height: 20px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
        </svg>
        <span style="font-weight: 500;">Chat</span>
      </a>
      <a
        href="/models"
        style="display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 8px; text-decoration: none; margin-bottom: 4px; background: {isActive('/models') ? '#282828' : 'transparent'}; color: {isActive('/models') ? 'white' : '#888'};"
      >
        <svg style="width: 20px; height: 20px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
        <span style="font-weight: 500;">Models</span>
      </a>
      <a
        href="/settings"
        style="display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 8px; text-decoration: none; margin-bottom: 4px; background: {isActive('/settings') ? '#282828' : 'transparent'}; color: {isActive('/settings') ? 'white' : '#888'};"
      >
        <svg style="width: 20px; height: 20px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        <span style="font-weight: 500;">Settings</span>
      </a>
      <a
        href="/hardware"
        style="display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 8px; text-decoration: none; margin-bottom: 4px; background: {isActive('/hardware') ? '#282828' : 'transparent'}; color: {isActive('/hardware') ? 'white' : '#888'};"
      >
        <svg style="width: 20px; height: 20px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
        </svg>
        <span style="font-weight: 500;">Hardware</span>
      </a>
    </nav>

    <!-- Bottom Info - Resource Gauges -->
    <div style="padding: 12px; border-top: 1px solid #282828;">
      {#if resources}
        <ResourceGauge
          label="CPU"
          used={resources.cpu_usage_percent}
          total={100}
          unit="%"
        />
        <ResourceGauge
          label="RAM"
          used={(resources.total_ram_bytes - resources.available_ram_bytes) / 1_000_000_000}
          total={resources.total_ram_bytes / 1_000_000_000}
        />
        {#if resources.selected_device && resources.selected_device.toUpperCase().includes('GPU') && resources.gpu_resources.length > 0}
          <ResourceGauge
            label="GPU"
            used={(resources.gpu_resources[0].total_vram_bytes - resources.gpu_resources[0].available_vram_bytes) / 1_000_000_000}
            total={resources.gpu_resources[0].total_vram_bytes / 1_000_000_000}
          />
        {/if}
      {:else}
        <p style="font-size: 10px; color: #666; text-align: center;">Loading...</p>
      {/if}
    </div>
  </aside>

  <!-- Main Content -->
  <main style="flex: 1; display: flex; flex-direction: column; background: #121212; overflow: hidden;">
    {@render children()}
  </main>
</div>

<style>
  a:hover {
    background: #1a1a1a !important;
    color: white !important;
  }
</style>
