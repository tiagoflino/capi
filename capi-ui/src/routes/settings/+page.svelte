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
      alert('Settings saved successfully');
    } catch (e) {
      alert('Failed to save settings: ' + e);
    }
    saving = false;
  }
</script>

<div class="settings-page">
  <div class="content-container">
    <header class="section-header">
      <h1>Preferences</h1>
      <p>Configure the engine and network interfaces</p>
    </header>

    {#if loading}
      <div class="placeholder-card loading">
        <div class="spinner"></div>
        <p>Loading configuration...</p>
      </div>
    {:else if config}
      <div class="card settings-card">
        <div class="settings-grid">
          <!-- Network Section -->
          <section class="settings-group">
            <h2 class="group-title">Inference Server</h2>
            <div class="field">
              <label for="server-host">Binding Host</label>
              <input id="server-host" type="text" bind:value={config.server_host} placeholder="0.0.0.0" />
            </div>
            <div class="field">
              <label for="server-port">Service Port</label>
              <input id="server-port" type="number" bind:value={config.server_port} />
            </div>
          </section>

          <!-- Engine Section -->
          <section class="settings-group">
            <h2 class="group-title">Hardware Orchestration</h2>
            <div class="field">
              <label for="device-pref">Preferred Compute Device</label>
              <div class="select-wrap">
                <select id="device-pref" bind:value={config.device_preference}>
                  <option value="auto">Automatic (Dynamic)</option>
                  <option value="cpu">CPU (Standard)</option>
                  <option value="gpu">GPU (Accelerated)</option>
                  <option value="npu">NPU (Dedicated AI)</option>
                </select>
                <div class="chevron"></div>
              </div>
            </div>

            <div class="field">
              <label for="resource-mode">Memory Allocation Policy</label>
              <div class="select-wrap">
                <select id="resource-mode" bind:value={config.resource_mode}>
                  <option value="strict">Strict (Block if low memory)</option>
                  <option value="loose">Flexible (Allow swapping)</option>
                </select>
                <div class="chevron"></div>
              </div>
            </div>
          </section>

          <!-- Intelligence Section -->
          <section class="settings-group">
            <h2 class="group-title">Neural Engine</h2>
            <div class="field">
              <label for="context-length">Default Context Horizon</label>
              <input id="context-length" type="number" bind:value={config.default_context_length} step="1024" />
              <span class="subfield-hint">Current: {(config.default_context_length / 1024).toFixed(0)}K tokens</span>
            </div>

            <div class="row-field">
              <div class="check-wrap">
                <input type="checkbox" bind:checked={config.auto_start} id="auto-start" />
                <label for="auto-start">Initialize server on application launch</label>
              </div>
            </div>
          </section>

          <button
            onclick={saveConfig}
            disabled={saving}
            class="save-btn"
          >
            {saving ? 'Syncing...' : 'Apply Changes'}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-page { height: 100%; overflow-y: auto; background: #fcfaf7; padding-bottom: 100px; }
  .content-container { max-width: 800px; margin: 0 auto; padding: 40px 24px; }

  .section-header { margin-bottom: 40px; }
  .section-header h1 { font-size: 32px; font-weight: 800; color: #3d3b38; letter-spacing: -0.03em; margin-bottom: 8px; }
  .section-header p { color: #8c8984; font-size: 16px; font-weight: 500; }

  .card { background: white; border: 1px solid rgba(0,0,0,0.05); border-radius: 24px; padding: 32px; box-shadow: 0 4px 20px rgba(0,0,0,0.02); }

  .settings-grid { display: flex; flex-direction: column; gap: 40px; }
  .settings-group { display: flex; flex-direction: column; gap: 20px; }
  .group-title { font-size: 13px; font-weight: 800; color: #b87333; text-transform: uppercase; letter-spacing: 0.1em; border-bottom: 1px solid #fae8d1; padding-bottom: 12px; margin-bottom: 8px; }

  .field { display: flex; flex-direction: column; gap: 8px; }
  .field label { font-size: 14px; font-weight: 700; color: #3d3b38; }
  .field input, .field select { padding: 12px 16px; background: #faf8f5; border: 1px solid rgba(0,0,0,0.05); border-radius: 12px; outline: none; font-size: 14px; color: #3d3b38; transition: all 0.2s; }
  .field input:focus, .field select:focus { border-color: #b87333; background: white; box-shadow: 0 0 0 4px rgba(184, 115, 51, 0.1); }

  .subfield-hint { font-size: 12px; color: #8c8984; font-weight: 600; margin-top: 4px; }

  .row-field { margin-top: 8px; }
  .check-wrap { display: flex; align-items: center; gap: 12px; }
  .check-wrap input[type="checkbox"] { width: 20px; height: 20px; border-radius: 6px; cursor: pointer; accent-color: #b87333; }
  .check-wrap label { font-size: 14px; font-weight: 600; color: #3d3b38; cursor: pointer; }

  .select-wrap { position: relative; }
  .select-wrap select { width: 100%; appearance: none; }
  .chevron { position: absolute; right: 16px; top: 50%; transform: translateY(-50%); width: 0; height: 0; border-left: 5px solid transparent; border-right: 5px solid transparent; border-top: 5px solid #8c8984; pointer-events: none; }

  .save-btn { margin-top: 20px; padding: 16px; background: #3d3b38; color: white; border: none; border-radius: 16px; font-size: 15px; font-weight: 800; cursor: pointer; transition: all 0.2s; }
  .save-btn:hover:not(:disabled) { background: #000; transform: translateY(-2px); box-shadow: 0 8px 24px rgba(0,0,0,0.1); }
  .save-btn:disabled { opacity: 0.3; cursor: not-allowed; }

  .placeholder-card { text-align: center; padding: 60px; background: white; border-radius: 24px; border: 1px solid rgba(0,0,0,0.05); }
  .spinner { width: 32px; height: 32px; border: 3px solid rgba(184, 115, 51, 0.1); border-top-color: #b87333; border-radius: 50%; margin: 0 auto 20px; animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
