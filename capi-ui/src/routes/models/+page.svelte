<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from 'svelte';

  interface Model {
    id: string;
    name: string;
    quantization?: string;
    size_bytes?: number;
    estimated_memory_bytes?: number;
  }

  interface HFModel {
    id: string;
    downloads: number;
  }

  let models = $state<Model[]>([]);
  let loading = $state(true);
  let searchQuery = $state('');
  let searchResults = $state<HFModel[]>([]);
  let searching = $state(false);

  onMount(async () => {
    await loadModels();
  });

  async function loadModels() {
    loading = true;
    try {
      models = await invoke('list_models');
    } catch (e) {
      console.error('Failed to load models:', e);
    }
    loading = false;
  }

  async function searchModels() {
    if (!searchQuery.trim()) return;

    searching = true;
    try {
      searchResults = await invoke('search_models', { query: searchQuery });
    } catch (e) {
      console.error('Search failed:', e);
    }
    searching = false;
  }

  async function downloadModel(modelId: string) {
    try {
      await invoke('download_model', { modelId });
      await loadModels();
    } catch (e) {
      alert('Download failed: ' + e);
    }
  }

  function formatSize(bytes: number | null | undefined) {
    if (!bytes) return '-';
    if (bytes > 1_000_000_000) {
      return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
    }
    return `${(bytes / 1_000_000).toFixed(0)} MB`;
  }
</script>

<div style="height: 100%; overflow-y: auto;">
  <div style="padding: 32px; max-width: 1200px; margin: 0 auto;">
    <h1 style="font-size: 32px; font-weight: bold; margin-bottom: 32px;">Models</h1>

    <!-- Installed Models -->
    <section style="margin-bottom: 48px;">
      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;">
        <h2 style="font-size: 20px; font-weight: 600; color: #e5e5e5;">Installed Models</h2>
        <button onclick={loadModels} style="font-size: 13px; color: #888; background: none; border: none; cursor: pointer;">Refresh</button>
      </div>

      {#if loading}
        <div style="background: #181818; border-radius: 12px; padding: 48px; text-align: center;">
          <div style="display: inline-block; width: 32px; height: 32px; border: 4px solid #282828; border-top-color: #3b82f6; border-radius: 50%; animation: spin 1s linear infinite;"></div>
          <p style="color: #888; margin-top: 16px;">Loading models...</p>
        </div>
      {:else if models.length === 0}
        <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 48px; text-align: center;">
          <svg style="width: 64px; height: 64px; margin: 0 auto 16px; color: #404040;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
          </svg>
          <p style="color: #888; margin-bottom: 8px;">No models installed</p>
          <p style="font-size: 14px; color: #666;">Search and download models below</p>
        </div>
      {:else}
        <div style="display: flex; flex-direction: column; gap: 8px;">
          {#each models as model}
            <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 16px; transition: background 0.2s;">
              <div style="display: flex; align-items: center; justify-between;">
                <div style="flex: 1;">
                  <h3 style="font-weight: 500; color: white; margin-bottom: 6px; font-size: 15px;">{model.name}</h3>
                  <div style="display: flex; align-items: center; gap: 12px; font-size: 13px; color: #888;">
                    <span>{model.quantization || '-'}</span>
                    <span>•</span>
                    <span>{formatSize(model.size_bytes)}</span>
                    {#if model.estimated_memory_bytes}
                      <span>•</span>
                      <span style="color: #60a5fa;">~{formatSize(model.estimated_memory_bytes)} RAM</span>
                    {/if}
                  </div>
                </div>
                <button style="padding: 8px 16px; font-size: 13px; color: #f87171; background: transparent; border: none; border-radius: 8px; cursor: pointer;">
                  Remove
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Search Models -->
    <section>
      <h2 style="font-size: 20px; font-weight: 600; color: #e5e5e5; margin-bottom: 16px;">Discover Models</h2>

      <div style="display: flex; gap: 12px; margin-bottom: 24px;">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search HuggingFace models..."
          style="flex: 1; padding: 12px 16px; background: #282828; border: 1px solid #404040; border-radius: 12px; color: white; font-size: 14px;"
          onkeydown={(e) => e.key === 'Enter' && searchModels()}
        />
        <button
          onclick={searchModels}
          disabled={searching || !searchQuery.trim()}
          style="padding: 12px 32px; background: linear-gradient(135deg, #3b82f6, #8b5cf6); border: none; color: white; font-weight: 600; border-radius: 12px; cursor: pointer; font-size: 14px; opacity: {searching || !searchQuery.trim() ? '0.3' : '1'};"
        >
          {searching ? 'Searching...' : 'Search'}
        </button>
      </div>

      {#if searchResults.length > 0}
        <div style="display: flex; flex-direction: column; gap: 8px;">
          {#each searchResults as result}
            <div style="background: #181818; border: 1px solid #282828; border-radius: 12px; padding: 16px; transition: background 0.2s;">
              <div style="display: flex; align-items: center; justify-content: space-between;">
                <div style="flex: 1;">
                  <h3 style="font-weight: 500; color: white; margin-bottom: 6px; font-size: 15px;">{result.id}</h3>
                  <p style="font-size: 13px; color: #888;">↓ {result.downloads.toLocaleString()} downloads</p>
                </div>
                <button
                  onclick={() => downloadModel(result.id)}
                  style="padding: 10px 24px; background: #16a34a; color: white; font-weight: 600; border: none; border-radius: 12px; cursor: pointer; font-size: 14px;"
                >
                  Download
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  button:hover:not(:disabled) {
    opacity: 0.9;
  }

  div:has(> div > h3):hover {
    background: #282828 !important;
  }
</style>
