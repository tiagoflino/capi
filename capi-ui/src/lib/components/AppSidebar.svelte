<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { selectedModel, currentSessionId, isSidebarOpen, isGenerating, inferenceMetrics, triggerNewChat } from '$lib/stores/app';
  import { page } from '$app/stores';
  import ResourceGauge from '$lib/ResourceGauge.svelte';

  let models = $state<any[]>([]);
  let sessions = $state<any[]>([]);
  let resources = $state<any>(null);
  let modelLoading = $state(false);
  let loadingStatus = $state('');

  // Subscribe to changes that require refreshing lists
  $effect(() => {
    // If generation stops, we might want to refresh sessions order
    if (!$isGenerating) {
      loadSessions();
    }
  });

  onMount(async () => {
    await loadModels();
    await loadSessions();
    updateResources();
    setInterval(updateResources, 2000);
  });

  async function loadModels() {
    try {
      models = await invoke('list_models');
      if (models.length > 0 && !$selectedModel) {
        $selectedModel = models[0].id; // Default to first model
      }
    } catch (e) {
      console.error('Failed to list models:', e);
    }
  }

  async function loadSessions() {
    try {
      sessions = await invoke('get_chat_sessions');
    } catch (e) {
      console.error('Failed to load sessions:', e);
    }
  }

  async function updateResources() {
    try {
      resources = await invoke('get_system_resources');
    } catch (e) {
      console.error('Resource update failed:', e);
    }
  }

  async function handleNewChat() {
    if ($isGenerating || !$selectedModel) return;
    try {
      // Create session on server
      const id = await invoke('create_chat_session', { 
        modelId: $selectedModel,
        title: 'New Chat'
      });
      await loadSessions();
      $currentSessionId = id as string;
      triggerNewChat.update(n => n + 1); // Tell page to clear
    } catch (e) {
      console.error('New chat failed:', e);
    }
  }

  async function deleteSession(id: string) {
    try {
      await invoke('delete_chat_session', { sessionId: id });
      if ($currentSessionId === id) {
        $currentSessionId = null;
      }
      await loadSessions();
    } catch (e) {
      console.error('Delete session failed:', e);
    }
  }

  async function preloadModel() {
    if (!$selectedModel) return;
    modelLoading = true;
    loadingStatus = 'Loading...';
    try {
      await invoke('load_model_direct', { modelId: $selectedModel });
      loadingStatus = 'Ready';
      setTimeout(() => loadingStatus = '', 2000);
    } catch (e) {
      loadingStatus = 'Error';
      console.error(e);
    }
    modelLoading = false;
  }

  const isActive = (path: string) => $page.url.pathname === path;
</script>

<aside class="sidebar glass" class:collapsed={!$isSidebarOpen}>
  <!-- 1. Header & Nav -->
  <div class="sidebar-header">
    <div class="brand">
      <div class="logo">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
           <circle cx="12" cy="12" r="10" stroke="#b87333" stroke-dasharray="4 4"/>
           <path d="M12 8v8M8 12h8" stroke="#b87333" stroke-linecap="round"/>
        </svg>
      </div>
      <span class="brand-text">Capi</span>
    </div>
    <button class="toggle-btn" onclick={() => $isSidebarOpen = !$isSidebarOpen}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M4 6h16M4 12h16M4 18h16" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  {#if $isSidebarOpen}
    <nav class="main-nav">
      <a href="/" class="nav-item {isActive('/') ? 'active' : ''}">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
        <span>Chat</span>
      </a>
      <a href="/models" class="nav-item {isActive('/models') ? 'active' : ''}">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/><line x1="3.27 6.96" y1="21 16" x2="3.27" y2="6.96"/></svg>
        <span>Models</span>
      </a>
      <a href="/hardware" class="nav-item {isActive('/hardware') ? 'active' : ''}">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="2" ry="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/></svg>
        <span>Hardware</span>
      </a>
      <a href="/settings" class="nav-item {isActive('/settings') ? 'active' : ''}">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
        <span>Settings</span>
      </a>
    </nav>
    <div class="divider"></div>
  {/if}

  <!-- 2. Chat History List (Only in Chat) -->
  {#if $isSidebarOpen && isActive('/')}
    <div class="section-label">History</div>
    <div class="chat-list">
      <button onclick={handleNewChat} disabled={$isGenerating} class="new-chat-row">
        <span class="icon">+</span>
        <span>New Chat</span>
      </button>

      {#each sessions as session}
        <div class="session-row {$currentSessionId === session.id ? 'active' : ''}" onclick={() => $currentSessionId = session.id}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chat-icon"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          <span class="title">{session.title || 'Untitled'}</span>
          <button class="delete-btn" onclick={(e) => { e.stopPropagation(); deleteSession(session.id); }}>×</button>
        </div>
      {/each}
    </div>
  {/if}

  <!-- 3. Bottom Controls -->
  {#if $isSidebarOpen}
    <div class="bottom-panel">
      <!-- Model Selector -->
      <div class="control-group">
        <label>Active Model</label>
        <div class="model-row">
          <select bind:value={$selectedModel} disabled={$isGenerating}>
             {#if models.length === 0}
               <option value="">No models</option>
             {:else}
               {#each models as model}
                 <option value={model.id}>{model.name}</option>
               {/each}
             {/if}
          </select>
          <button onclick={preloadModel} disabled={modelLoading || !$selectedModel} class="load-btn" title="Load Model">
             {#if modelLoading}
               <div class="spinner"></div>
             {:else}
               ⚡
             {/if}
          </button>
        </div>
        {#if loadingStatus}
          <div class="status-tiny">{loadingStatus}</div>
        {/if}
      </div>
      
      <!-- Metrics -->
      {#if resources}
        <div class="control-group metrics-group">
          <label>System</label>
          <ResourceGauge label="CPU" used={resources.cpu_usage_percent} total={100} unit="%" />
          <ResourceGauge label="RAM" used={(resources.total_ram_bytes - resources.available_ram_bytes)/1e9} total={resources.total_ram_bytes/1e9} />
          {#if resources.gpu_resources.length > 0}
            <ResourceGauge label="GPU" used={resources.gpu_resources[0].usage_percent} total={100} unit="%" />
             <!-- VRAM -->
             <ResourceGauge label="VRAM" used={(resources.gpu_resources[0].total_vram_bytes - resources.gpu_resources[0].available_vram_bytes)/1e9} total={resources.gpu_resources[0].total_vram_bytes/1e9} />
          {/if}
        </div>
      {/if}

       <!-- Inference Metrics (Live) -->
       {#if $inferenceMetrics.tokens_per_second > 0}
          <div class="control-group perf-group">
            <div class="perf-row"><span>Speed</span> <strong>{$inferenceMetrics.tokens_per_second.toFixed(1)} t/s</strong></div>
            <div class="perf-row"><span>Context</span> <strong>{$inferenceMetrics.total_context_tokens}</strong></div>
          </div>
       {/if}
    </div>
  {/if}
</aside>

<style>
  .sidebar {
    width: 260px;
    background: #ffffff;
    border-right: 1px solid rgba(0,0,0,0.06);
    display: flex;
    flex-direction: column;
    transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    position: relative;
    z-index: 100;
  }

  .sidebar.collapsed {
    width: 60px;
    align-items: center;
  }

  .sidebar-header {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    flex-shrink: 0;
  }
  
  .collapsed .sidebar-header {
     justify-content: center;
     padding: 0;
  }
  .collapsed .brand { display: none; }

  .brand { display: flex; align-items: center; gap: 10px; font-weight: 700; color: #3d3b38; }
  .logo { width: 24px; height: 24px; color: #b87333; }
  .toggle-btn { background: none; border: none; color: #8c8984; cursor: pointer; padding: 4px; border-radius: 6px; }
  .toggle-btn:hover { background: rgba(0,0,0,0.05); color: #3d3b38; }

  .main-nav { padding: 8px 12px; display: flex; flex-direction: column; gap: 4px; flex-shrink: 0; }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-radius: 8px;
    color: #8c8984;
    text-decoration: none;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s;
  }
  .nav-item:hover { background: rgba(0,0,0,0.03); color: #3d3b38; }
  .nav-item.active { background: #fae8d1; color: #b87333; font-weight: 600; }

  .divider { height: 1px; background: rgba(0,0,0,0.06); margin: 8px 16px; flex-shrink: 0; }

  .section-label { 
    padding: 12px 24px 8px; 
    font-size: 11px; 
    font-weight: 700; 
    text-transform: uppercase; 
    letter-spacing: 0.05em; 
    color: #8c8984;
  }

  .chat-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .new-chat-row {
     display: flex;
     align-items: center;
     gap: 10px;
     padding: 10px 12px;
     border: 1px dashed rgba(0,0,0,0.1);
     border-radius: 8px;
     background: transparent;
     color: #b87333;
     font-size: 13px;
     font-weight: 600;
     cursor: pointer;
     transition: all 0.2s;
     margin-bottom: 8px;
  }
  .new-chat-row:hover:not(:disabled) { background: #fffbf6; border-color: #b87333; }

  .session-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: 8px;
    color: #555;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
  }
  .session-row:hover { background: rgba(0,0,0,0.03); }
  .session-row.active { background: #f2f2f7; color: #3d3b38; font-weight: 600; }
  
  .chat-icon { opacity: 0.5; width: 14px; height: 14px; flex-shrink: 0; }
  .title { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .delete-btn { opacity: 0; background: none; border: none; color: #ff453a; font-size: 16px; cursor: pointer; padding: 0 4px; }
  .session-row:hover .delete-btn { opacity: 1; }

  .bottom-panel {
    border-top: 1px solid rgba(0,0,0,0.06);
    background: #faf9f7;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    flex-shrink: 0;
  }

  .control-group label {
    display: block;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    color: #8c8984;
    margin-bottom: 8px;
  }

  .model-row { display: flex; gap: 8px; }
  select { flex: 1; padding: 6px; border-radius: 6px; border: 1px solid rgba(0,0,0,0.1); font-size: 12px; }
  .load-btn { width: 32px; background: #3d3b38; color: white; border: none; border-radius: 6px; cursor: pointer; display: flex; align-items: center; justify-content: center; }
  .load-btn:hover:not(:disabled) { background: #000; }
  .load-btn:disabled { opacity: 0.3; }

  .metrics-group { display: flex; flex-direction: column; gap: 6px; }
  
  .perf-group { background: #fff; padding: 8px; border-radius: 8px; border: 1px solid rgba(0,0,0,0.05); }
  .perf-row { display: flex; justify-content: space-between; font-size: 11px; color: #8c8984; }
  .perf-row strong { color: #b87333; }

  .spinner { width: 12px; height: 12px; border: 2px solid rgba(255,255,255,0.3); border-top-color: white; border-radius: 50%; animation: spin 1s linear infinite; }
  .status-tiny { font-size: 10px; color: #b87333; margin-top: 4px; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
