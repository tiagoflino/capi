<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from 'svelte';

  interface Config {
    server_host: string;
    server_port: number;
    device_preference: string;
    resource_mode: string;
    default_context_length: number;
    auto_start: boolean;
  }

  let config = $state<Config | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  onMount(async () => {
    await loadConfig();
  });

  async function loadConfig() {
    loading = true;
    try {
      config = await invoke('get_config');
    } catch (e) {
      console.error('Failed to load config:', e);
    }
    loading = false;
  }

  async function saveConfig() {
    saving = true;
    try {
      await invoke('save_config', { config });
      alert('Settings saved');
    } catch (e) {
      alert('Failed to save: ' + e);
    }
    saving = false;
  }
</script>

<div style="height: 100%; overflow-y: auto;">
  <div style="padding: 32px; max-w: 800px; margin: 0 auto;">
    <h1 style="font-size: 32px; font-weight: bold; margin-bottom: 32px;">Settings</h1>

    {#if loading}
      <div style="text-align: center; padding: 48px;">
        <div style="display: inline-block; width: 32px; height: 32px; border: 4px solid #282828; border-top-color: #3b82f6; border-radius: 50%; animation: spin 1s linear infinite;"></div>
        <p style="color: #888; margin-top: 16px;">Loading...</p>
      </div>
    {:else if config}
      <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 32px;">
        <div style="display: flex; flex-direction: column; gap: 32px;">
          <!-- Server Settings -->
          <div>
            <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 16px;">Server</h2>
            <div style="display: flex; flex-direction: column; gap: 16px;">
              <div>
                <label for="server-host" style="display: block; font-size: 13px; font-weight: 500; color: #aaa; margin-bottom: 6px;">Host</label>
                <input id="server-host" type="text" bind:value={config.server_host} style="width: 100%; padding: 10px 14px; background: #282828; border: 1px solid #404040; border-radius: 8px; color: white; font-size: 14px;" />
              </div>
              <div>
                <label for="server-port" style="display: block; font-size: 13px; font-weight: 500; color: #aaa; margin-bottom: 6px;">Port</label>
                <input id="server-port" type="number" bind:value={config.server_port} style="width: 100%; padding: 10px 14px; background: #282828; border: 1px solid #404040; border-radius: 8px; color: white; font-size: 14px;" />
              </div>
            </div>
          </div>

          <!-- Device -->
          <div>
            <label for="device-pref" style="display: block; font-size: 13px; font-weight: 500; color: #aaa; margin-bottom: 6px;">Device Preference</label>
            <select id="device-pref" bind:value={config.device_preference} style="width: 100%; padding: 10px 14px; background: #282828; border: 1px solid #404040; border-radius: 8px; color: white; font-size: 14px;">
              <option value="auto">Auto</option>
              <option value="cpu">CPU</option>
              <option value="gpu">GPU</option>
              <option value="npu">NPU</option>
            </select>
          </div>

          <!-- Resource Mode -->
          <div>
            <label for="resource-mode" style="display: block; font-size: 13px; font-weight: 500; color: #aaa; margin-bottom: 6px;">Resource Mode</label>
            <select id="resource-mode" bind:value={config.resource_mode} style="width: 100%; padding: 10px 14px; background: #282828; border: 1px solid #404040; border-radius: 8px; color: white; font-size: 14px;">
              <option value="strict">Strict (block if insufficient memory)</option>
              <option value="loose">Loose (warn but allow)</option>
            </select>
          </div>

          <!-- Context Length -->
          <div>
            <label for="context-length" style="display: block; font-size: 13px; font-weight: 500; color: #aaa; margin-bottom: 6px;">Default Context Length</label>
            <input id="context-length" type="number" bind:value={config.default_context_length} step="1024" style="width: 100%; padding: 10px 14px; background: #282828; border: 1px solid #404040; border-radius: 8px; color: white; font-size: 14px;" />
            <p style="font-size: 12px; color: #666; margin-top: 6px;">{(config.default_context_length / 1024).toFixed(0)}K tokens</p>
          </div>

          <!-- Auto Start -->
          <div style="display: flex; align-items: center; gap: 12px;">
            <input type="checkbox" bind:checked={config.auto_start} id="auto-start" style="width: 18px; height: 18px; cursor: pointer;" />
            <label for="auto-start" style="font-size: 14px; font-weight: 500; color: #e5e5e5; cursor: pointer;">Auto-start server</label>
          </div>

          <button
            onclick={saveConfig}
            disabled={saving}
            style="width: 100%; padding: 12px; background: linear-gradient(135deg, #3b82f6, #8b5cf6); border: none; color: white; font-weight: 600; border-radius: 12px; cursor: pointer; font-size: 14px; opacity: {saving ? '0.5' : '1'};"
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  button:hover:not(:disabled) {
    opacity: 0.9;
  }

  input:hover, select:hover {
    background: #2a2a2a;
  }
</style>
