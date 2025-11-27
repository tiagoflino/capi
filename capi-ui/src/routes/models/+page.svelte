<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from 'svelte';
  import Modal from '$lib/Modal.svelte';

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

  interface FileInfo {
    name: string;
    size?: number;
  }

  let models = $state<Model[]>([]);
  let loading = $state(true);
  let searchQuery = $state('');
  let searchResults = $state<HFModel[]>([]);
  let searching = $state(false);

  // Browse flow state
  let showQuantModal = $state(false);
  let showFileModal = $state(false);
  let selectedBaseModel = $state<HFModel | null>(null);
  let quantizedVersions = $state<HFModel[]>([]);
  let selectedQuantModel = $state<HFModel | null>(null);
  let modelFiles = $state<FileInfo[]>([]);
  let loadingQuants = $state(false);
  let loadingFiles = $state(false);
  let downloading = $state(false);
  let downloadProgress = $state(0);

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

  async function browseModel(model: HFModel) {
    selectedBaseModel = model;
    loadingQuants = true;
    showQuantModal = true;

    try {
      quantizedVersions = await invoke('find_quantized_versions', {
        baseModelId: model.id
      });
    } catch (e) {
      console.error('Failed to find quantized versions:', e);
      quantizedVersions = [];
    }

    loadingQuants = false;
  }

  async function selectQuantVersion(quantModel: HFModel) {
    selectedQuantModel = quantModel;
    showQuantModal = false;
    showFileModal = true;
    loadingFiles = true;

    try {
      const files: FileInfo[] = await invoke('fetch_model_files', {
        modelId: quantModel.id
      });

      modelFiles = files.filter(f => f.name.endsWith('.gguf'));
    } catch (e) {
      console.error('Failed to fetch files:', e);
      modelFiles = [];
    }

    loadingFiles = false;
  }

  async function downloadFile(filename: string) {
    if (!selectedQuantModel) return;

    downloading = true;
    downloadProgress = 0;

    // Listen for download progress
    const unlisten = await listen('download-progress', (event: any) => {
      downloadProgress = event.payload.percent;
    });

    try {
      await invoke('download_specific_file', {
        modelId: selectedQuantModel.id,
        filename
      });

      showFileModal = false;
      await loadModels();
      downloadProgress = 0;
    } catch (e) {
      alert('Download failed: ' + e);
    } finally {
      unlisten();
      downloading = false;
    }
  }

  function extractQuantization(filename: string): string | null {
    const patterns = ['Q2_K', 'Q3_K_S', 'Q3_K_M', 'Q3_K_L', 'Q4_0', 'Q4_1', 'Q4_K_S', 'Q4_K_M',
                      'Q5_0', 'Q5_1', 'Q5_K_S', 'Q5_K_M', 'Q6_K', 'Q8_0', 'F16', 'F32'];

    const upper = filename.toUpperCase();
    for (const pattern of patterns) {
      if (upper.includes(pattern)) return pattern;
    }
    return null;
  }

  async function removeModel(modelId: string) {
    if (!confirm(`Remove model ${modelId}?`)) return;

    try {
      await invoke('remove_model', { modelId });
      await loadModels();
    } catch (e) {
      alert('Remove failed: ' + e);
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
                <button
                  onclick={() => removeModel(model.id)}
                  style="padding: 8px 16px; font-size: 13px; color: #f87171; background: transparent; border: none; border-radius: 8px; cursor: pointer;"
                >
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
                  onclick={() => browseModel(result)}
                  style="padding: 10px 24px; background: #16a34a; color: white; font-weight: 600; border: none; border-radius: 12px; cursor: pointer; font-size: 14px;"
                >
                  Browse
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>

<!-- Quantized Versions Modal -->
<Modal isOpen={showQuantModal} onClose={() => showQuantModal = false} title="Select Quantized Version">
  {#snippet children()}
    {#if loadingQuants}
      <div style="text-align: center; padding: 48px;">
        <div style="display: inline-block; width: 32px; height: 32px; border: 4px solid #282828; border-top-color: #3b82f6; border-radius: 50%; animation: spin 1s linear infinite;"></div>
        <p style="color: #888; margin-top: 16px;">Finding quantized versions...</p>
      </div>
    {:else if quantizedVersions.length === 0}
      <div style="text-align: center; padding: 48px;">
        <p style="color: #888;">No quantized GGUF versions found</p>
        <p style="font-size: 13px; color: #666; margin-top: 8px;">Try using the base model directly or search for a different model</p>
      </div>
    {:else}
      <div style="display: flex; flex-direction: column; gap: 8px;">
        {#each quantizedVersions as quant}
          <button
            onclick={() => selectQuantVersion(quant)}
            style="text-align: left; background: #181818; border: 1px solid #282828; border-radius: 8px; padding: 16px; cursor: pointer; transition: background 0.2s;"
          >
            <h3 style="font-weight: 500; color: white; margin-bottom: 6px; font-size: 15px;">{quant.id}</h3>
            <p style="font-size: 13px; color: #888;">↓ {quant.downloads.toLocaleString()} downloads</p>
          </button>
        {/each}
      </div>
    {/if}
  {/snippet}
</Modal>

<!-- GGUF Files Modal -->
<Modal isOpen={showFileModal} onClose={() => showFileModal = false} title="Select Quantization">
  {#snippet children()}
    {#if downloading}
      <div style="text-align: center; padding: 48px;">
        <h3 style="font-size: 18px; font-weight: 600; margin-bottom: 24px;">Downloading...</h3>
        <div style="width: 100%; max-width: 400px; margin: 0 auto;">
          <div style="height: 8px; background: #282828; border-radius: 4px; overflow: hidden;">
            <div style="width: {downloadProgress}%; height: 100%; background: linear-gradient(90deg, #3b82f6, #8b5cf6); transition: width 0.3s;"></div>
          </div>
          <p style="color: #888; margin-top: 12px; font-size: 14px;">{downloadProgress.toFixed(1)}%</p>
        </div>
      </div>
    {:else if loadingFiles}
      <div style="text-align: center; padding: 48px;">
        <div style="display: inline-block; width: 32px; height: 32px; border: 4px solid #282828; border-top-color: #3b82f6; border-radius: 50%; animation: spin 1s linear infinite;"></div>
        <p style="color: #888; margin-top: 16px;">Loading files...</p>
      </div>
    {:else if modelFiles.length === 0}
      <div style="text-align: center; padding: 48px;">
        <p style="color: #888;">No GGUF files found</p>
      </div>
    {:else}
      <div style="display: flex; flex-direction: column; gap: 8px;">
        {#each modelFiles as file}
          <button
            onclick={() => downloadFile(file.name)}
            disabled={downloading}
            style="text-align: left; background: #181818; border: 1px solid #282828; border-radius: 8px; padding: 16px; cursor: pointer; transition: background 0.2s; opacity: {downloading ? '0.5' : '1'};"
          >
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <div>
                <h3 style="font-weight: 500; color: white; font-size: 14px;">
                  {extractQuantization(file.name) || 'Unknown'} - {file.name}
                </h3>
                <p style="font-size: 13px; color: #888; margin-top: 4px;">
                  {file.size ? `${(file.size / 1_000_000_000).toFixed(1)} GB` : 'Size unknown'}
                </p>
              </div>
              <svg style="width: 20px; height: 20px; color: #16a34a;" fill="currentColor" viewBox="0 0 24 24">
                <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
              </svg>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  {/snippet}
</Modal>

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
