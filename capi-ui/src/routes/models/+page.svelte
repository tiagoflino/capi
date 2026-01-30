<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
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

<div class="models-page">
  <div class="content-container">
    <header class="section-header">
      <h1>Model Repository</h1>
      <p>Manage and discover large language models</p>
    </header>

    <!-- Installed Models -->
    <section class="models-section">
      <div class="section-top">
        <h2 class="section-title">Installed Assets</h2>
        <button onclick={loadModels} class="refresh-btn">Refresh Registry</button>
      </div>

      {#if loading}
        <div class="placeholder-card loading">
          <div class="spinner"></div>
          <p>Scanning local storage...</p>
        </div>
      {:else if models.length === 0}
        <div class="placeholder-card empty">
          <div class="empty-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
            </svg>
          </div>
          <p>No models indexed</p>
          <span class="sub-placeholder">Search and deploy models from HuggingFace below.</span>
        </div>
      {:else}
        <div class="model-list">
          {#each models as model}
            <div class="model-item">
              <div class="model-main">
                <h3>{model.name}</h3>
                <div class="model-meta">
                  <span class="tag">{model.quantization || 'GGUF'}</span>
                  <span class="dot"></span>
                  <span>{formatSize(model.size_bytes)}</span>
                  {#if model.estimated_memory_bytes}
                    <span class="dot"></span>
                    <span class="memory-estimate">~{formatSize(model.estimated_memory_bytes)} VRAM</span>
                  {/if}
                </div>
              </div>
              <button onclick={() => removeModel(model.id)} class="delete-btn">Eject</button>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Discovery Section -->
    <section class="discovery-section">
      <h2 class="section-title">Model Discovery</h2>

      <div class="search-bar">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search HuggingFace..."
          onkeydown={(e) => e.key === 'Enter' && searchModels()}
        />
        <button
          onclick={searchModels}
          disabled={searching || !searchQuery.trim()}
          class="search-btn"
        >
          {searching ? 'Probing...' : 'Search'}
        </button>
      </div>

      {#if searchResults.length > 0}
        <div class="discovery-list">
          {#each searchResults as result}
            <div class="discovery-item">
              <div class="discovery-main">
                <h3>{result.id}</h3>
                <p class="downloads-count">↓ {result.downloads.toLocaleString()} pull requests</p>
              </div>
              <button onclick={() => browseModel(result)} class="browse-btn">Deploy</button>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>

<Modal isOpen={showQuantModal} onClose={() => showQuantModal = false} title="Select Quantization Hierarchy">
  {#if loadingQuants}
    <div class="modal-loading">
      <div class="spinner"></div>
      <p>Fetching quantized artifacts...</p>
    </div>
  {:else if quantizedVersions.length === 0}
    <div class="modal-empty">
      <p>No valid GGUF artifacts found</p>
      <span>Try a different base model or quantization provider.</span>
    </div>
  {:else}
    <div class="selection-list">
      {#each quantizedVersions as quant}
        <button onclick={() => selectQuantVersion(quant)} class="selection-item">
          <div class="selection-info">
            <span class="s-id">{quant.id}</span>
            <span class="s-dl">↓ {quant.downloads.toLocaleString()} downloads</span>
          </div>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M9 5l7 7-7 7" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
      {/each}
    </div>
  {/if}
</Modal>

<Modal isOpen={showFileModal} onClose={() => showFileModal = false} title="Deployment Specification">
  {#if downloading}
    <div class="download-state">
      <h3 class="dl-title">Transferring Neural Weights</h3>
      <div class="progress-wrap">
        <div class="dl-track">
          <div class="dl-fill" style="width: {downloadProgress}%;"></div>
        </div>
        <span class="dl-percent">{downloadProgress.toFixed(1)}%</span>
      </div>
    </div>
  {:else if loadingFiles}
    <div class="modal-loading">
      <div class="spinner"></div>
      <p>Indexing remote weights...</p>
    </div>
  {:else if modelFiles.length === 0}
    <div class="modal-empty"><p>No compatible GGUF weights found</p></div>
  {:else}
    <div class="selection-list">
      {#each modelFiles as file}
        <button onclick={() => downloadFile(file.name)} disabled={downloading} class="selection-item compact">
          <div class="selection-info">
            <span class="s-quant">{extractQuantization(file.name) || 'Unknown'}</span>
            <span class="s-filename">{file.name}</span>
          </div>
          <div class="selection-right">
            <span class="s-size">{file.size ? formatSize(file.size) : '???'}</span>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M12 15l-4-4h8l-4 4zm0-12v12" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M5 19h14v2H5z" stroke-linecap="round"/>
            </svg>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</Modal>

<style>
  .models-page { height: 100%; overflow-y: auto; background: #fcfaf7; padding-bottom: 100px; }
  .content-container { max-width: 900px; margin: 0 auto; padding: 40px 24px; }

  .section-header { margin-bottom: 48px; }
  .section-header h1 { font-size: 32px; font-weight: 800; color: #3d3b38; letter-spacing: -0.03em; margin-bottom: 8px; }
  .section-header p { color: #8c8984; font-size: 16px; font-weight: 500; }

  .section-top { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
  .section-title { font-size: 20px; font-weight: 800; color: #3d3b38; letter-spacing: -0.02em; }
  .refresh-btn { background: none; border: none; color: #b87333; font-size: 12px; font-weight: 800; text-transform: uppercase; cursor: pointer; opacity: 0.7; transition: opacity 0.2s; }
  .refresh-btn:hover { opacity: 1; }

  .placeholder-card { background: white; border: 1px solid rgba(0,0,0,0.05); border-radius: 20px; padding: 60px 20px; text-align: center; }
  .empty-icon { width: 64px; height: 64px; margin: 0 auto 20px; color: #fae8d1; }
  .placeholder-card p { font-size: 16px; font-weight: 700; color: #3d3b38; margin-bottom: 4px; }
  .sub-placeholder { font-size: 14px; color: #8c8984; }

  .model-list { display: flex; flex-direction: column; gap: 12px; }
  .model-item { background: white; border: 1px solid rgba(0,0,0,0.05); border-radius: 16px; padding: 16px 20px; display: flex; align-items: center; justify-content: space-between; box-shadow: 0 4px 12px rgba(0,0,0,0.02); }
  .model-main h3 { font-size: 15px; font-weight: 800; color: #3d3b38; margin-bottom: 4px; }
  .model-meta { display: flex; align-items: center; gap: 8px; font-size: 12px; font-weight: 600; color: #8c8984; font-family: monospace; }
  .tag { background: #fae8d1; color: #b87333; padding: 2px 8px; border-radius: 6px; font-size: 10px; font-weight: 800; }
  .dot { width: 3px; height: 3px; background: #dcdad7; border-radius: 50%; }
  .memory-estimate { color: #369b7d; }
  .delete-btn { background: #fee2e2; color: #dc2626; border: none; padding: 6px 14px; border-radius: 10px; font-size: 11px; font-weight: 800; cursor: pointer; transition: all 0.2s; }
  .delete-btn:hover { background: #fecaca; transform: translateY(-1px); }

  .search-bar { display: flex; gap: 12px; margin: 24px 0 32px; }
  .search-bar input { flex: 1; padding: 14px 20px; background: white; border: 1px solid rgba(0,0,0,0.05); border-radius: 16px; font-size: 15px; outline: none; box-shadow: 0 4px 12px rgba(0,0,0,0.02); transition: border-color 0.2s; }
  .search-bar input:focus { border-color: #b87333; }
  .search-btn { background: #3d3b38; color: white; border: none; padding: 0 28px; border-radius: 16px; font-size: 14px; font-weight: 700; cursor: pointer; transition: all 0.2s; }
  .search-btn:hover:not(:disabled) { background: #000; transform: translateY(-1px); }
  .search-btn:disabled { opacity: 0.3; cursor: not-allowed; }

  .discovery-list { display: grid; grid-template-columns: 1fr; gap: 10px; }
  .discovery-item { background: white; border: 1px solid rgba(0,0,0,0.05); border-radius: 16px; padding: 16px 20px; display: flex; align-items: center; justify-content: space-between; }
  .discovery-main h3 { font-size: 14px; font-weight: 700; color: #3d3b38; margin-bottom: 2px; }
  .downloads-count { font-size: 11px; color: #8c8984; font-weight: 600; }
  .browse-btn { background: #3d3b38; color: white; border: none; padding: 8px 18px; border-radius: 10px; font-size: 12px; font-weight: 700; cursor: pointer; transition: all 0.2s; }
  .browse-btn:hover { background: #000; transform: scale(1.03); }

  .selection-list { display: flex; flex-direction: column; gap: 8px; }
  .selection-item { display: flex; align-items: center; justify-content: space-between; padding: 16px; background: #faf8f5; border: 1px solid rgba(0,0,0,0.05); border-radius: 14px; cursor: pointer; transition: all 0.2s; text-align: left; width: 100%; color: inherit; }
  .selection-item:hover { background: #fae8d1; border-color: #b87333; transform: translateX(4px); }
  .selection-info { display: flex; flex-direction: column; gap: 4px; }
  .s-id { font-size: 14px; font-weight: 700; color: #3d3b38; }
  .s-dl { font-size: 11px; color: #8c8984; }
  .s-quant { font-weight: 800; color: #b87333; font-size: 12px; font-family: monospace; }
  .s-filename { font-size: 12px; color: #8c8984; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 300px; }
  .selection-right { display: flex; align-items: center; gap: 12px; }
  .s-size { font-size: 12px; font-weight: 700; color: #3d3b38; }

  .modal-loading { text-align: center; padding: 40px 0; color: #8c8984; }
  .download-state { text-align: center; padding: 20px 0; }
  .dl-title { font-size: 18px; font-weight: 800; color: #3d3b38; margin-bottom: 24px; }
  .progress-wrap { display: flex; flex-direction: column; gap: 10px; align-items: center; }
  .dl-track { width: 100%; height: 8px; background: #faf8f5; border-radius: 4px; overflow: hidden; }
  .dl-fill { height: 100%; background: #b87333; border-radius: 4px; transition: width 0.3s; }
  .dl-percent { font-size: 14px; font-weight: 800; color: #b87333; }

  .spinner { width: 24px; height: 24px; border: 3px solid rgba(184, 115, 51, 0.1); border-top-color: #b87333; border-radius: 50%; margin: 0 auto 16px; animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
