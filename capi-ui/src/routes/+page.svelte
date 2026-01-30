<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from 'svelte';

  interface Model {
    id: string;
    name: string;
  }

  interface Message {
    role: string;
    content: string;
    metrics?: {
      tokens_per_second?: number;
      time_to_first_token_ms?: number;
      completion_tokens?: number;
    };
  }

  interface Config {
    server_url: string;
  }

  let models = $state<Model[]>([]);
  let selectedModel = $state('');
  let messages = $state<Message[]>([]);
  let input = $state('');
  let generating = $state(false);
  let loadingStatus = $state('');
  let modelLoading = $state(false);

  onMount(async () => {
    try {
      const modelList = await invoke('list_models');
      models = modelList;
      if (models.length > 0) {
        selectedModel = models[0].id;
        // Don't preload automatically - let user select when ready
      }
    } catch (e) {
      console.error('Failed to load models:', e);
    }
  });



  async function preloadModel(modelId: string) {
    if (!modelId) return;
    modelLoading = true;
    loadingStatus = 'Loading model into memory...';
    try {
      const result = await invoke('load_model_direct', { modelId });
      loadingStatus = 'Model ready';
      console.log(result);
      setTimeout(() => loadingStatus = '', 2000);
    } catch (e: any) {
      loadingStatus = `Error: ${e}`;
      console.error('Model load failed:', e);
    }
    modelLoading = false;
  }

  async function sendMessage() {
    if (!input.trim() || !selectedModel || generating) return;

    const userMessage = input;
    input = '';

    messages = [...messages, { role: 'user', content: userMessage }];
    generating = true;
    loadingStatus = 'Generating...';

    const messageIndex = messages.length;
    messages = [...messages, { role: 'assistant', content: '' }];

    // Listen for token events
    const unlisten = await listen('chat-token', (event: any) => {
      const token = event.payload.token;
      messages[messageIndex].content += token;
      messages = [...messages];
    });

    try {
      const metrics = await invoke('chat_direct', {
        modelId: selectedModel,
        prompt: userMessage,
      });

      messages[messageIndex].metrics = {
        tokens_per_second: metrics.tokens_per_second,
        time_to_first_token_ms: metrics.time_to_first_token_ms,
        completion_tokens: metrics.num_output_tokens,
      };
      messages = [...messages];
    } catch (e: any) {
      console.error('Chat failed:', e);
      messages[messageIndex] = {
        role: 'system',
        content: `⚠ Error: ${e}\n\nLoad the model first by selecting it from the dropdown.`
      };
      messages = [...messages];
    } finally {
      unlisten();
      generating = false;
      loadingStatus = '';
    }
  }

  function clearChat() {
    messages = [];
  }
</script>

<div style="display: flex; flex-direction: column; height: 100%; overflow: hidden;">
  <!-- Header -->
  <div style="padding: 16px 24px; border-bottom: 1px solid #282828; background: #181818;">
    <div style="display: flex; align-items: center; justify-content: space-between;">
      <div style="display: flex; align-items: center; gap: 16px;">
        <select
          bind:value={selectedModel}
          onchange={(e) => preloadModel(e.currentTarget.value)}
          disabled={generating || modelLoading}
          style="padding: 8px 16px; background: #282828; border: 1px solid #404040; border-radius: 8px; color: white; font-size: 14px; cursor: pointer;"
        >
          {#if models.length === 0}
            <option value="">No models available</option>
          {:else}
            {#each models as model}
              <option value={model.id}>{model.name}</option>
            {/each}
          {/if}
        </select>
        <button
          onclick={() => preloadModel(selectedModel)}
          disabled={!selectedModel || generating || modelLoading}
          style="padding: 8px 16px; background: linear-gradient(135deg, #3b82f6, #8b5cf6); border: none; color: white; font-weight: 600; border-radius: 8px; cursor: pointer; font-size: 13px; opacity: {!selectedModel || generating || modelLoading ? '0.3' : '1'};"
        >
          {modelLoading ? 'Loading...' : 'Load Model'}
        </button>
        {#if generating || modelLoading}
          <span style="font-size: 13px; color: #888;">{loadingStatus}</span>
        {/if}
      </div>
      <button
        onclick={clearChat}
        disabled={messages.length === 0}
        style="padding: 8px 16px; background: transparent; border: none; color: #888; font-size: 13px; border-radius: 8px; cursor: pointer;"
      >
        Clear
      </button>
    </div>
  </div>

  <!-- Messages -->
  <div style="flex: 1; overflow-y: auto; padding: 24px;">
    <div style="max-width: 800px; margin: 0 auto;">
      {#if messages.length === 0}
        <div style="text-align: center; margin-top: 120px;">
          <div style="width: 64px; height: 64px; background: linear-gradient(135deg, #3b82f6, #8b5cf6); border-radius: 50%; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;">
            <svg style="width: 32px; height: 32px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
          </div>
          <h2 style="font-size: 24px; font-weight: bold; margin-bottom: 8px;">Start a conversation</h2>
          <p style="color: #888; font-size: 14px;">Ask anything and get instant responses</p>
        </div>
      {:else}
        {#each messages as message}
          <div style="display: flex; justify-content: {message.role === 'user' ? 'flex-end' : 'flex-start'}; margin-bottom: 16px;">
            <div style="max-width: 70%; display: flex; align-items-start; gap: 12px; flex-direction: {message.role === 'user' ? 'row-reverse' : 'row'};">
              <div style="flex-shrink: 0; width: 32px; height: 32px; border-radius: 50%; background: {message.role === 'user' ? '#3b82f6' : message.role === 'system' ? '#ef4444' : 'linear-gradient(135deg, #3b82f6, #8b5cf6)'}; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 12px; font-weight: bold;">{message.role === 'user' ? 'U' : message.role === 'system' ? '!' : 'A'}</span>
              </div>
              <div style="flex: 1;">
                <div style="padding: 12px 16px; border-radius: 12px; background: {message.role === 'user' ? '#2563eb' : message.role === 'system' ? '#7f1d1d' : '#282828'}; border: {message.role === 'system' ? '1px solid #991b1b' : 'none'};">
                  <p style="font-size: 14px; line-height: 1.6; white-space: pre-wrap; color: {message.role === 'system' ? '#fecaca' : 'white'};">{message.content}</p>
                </div>
                {#if message.metrics && message.role === 'assistant'}
                  <div style="display: flex; gap: 16px; margin-top: 6px; padding-left: 16px; font-size: 11px; color: #666;">
                    {#if message.metrics.tokens_per_second}
                      <span>{message.metrics.tokens_per_second.toFixed(1)} tok/s</span>
                    {/if}
                    {#if message.metrics.time_to_first_token_ms}
                      <span>•</span>
                      <span>TTFT: {message.metrics.time_to_first_token_ms.toFixed(0)}ms</span>
                    {/if}
                    {#if message.metrics.completion_tokens}
                      <span>•</span>
                      <span>{message.metrics.completion_tokens} tokens</span>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>
          </div>
        {/each}

        {#if generating}
          <div style="display: flex; justify-content: flex-start; margin-bottom: 16px;">
            <div style="display: flex; align-items-start; gap: 12px;">
              <div style="flex-shrink: 0; width: 32px; height: 32px; border-radius: 50%; background: linear-gradient(135deg, #3b82f6, #8b5cf6); display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 12px; font-weight: bold;">A</span>
              </div>
              <div style="padding: 12px 16px; background: #282828; border-radius: 12px;">
                <div style="display: flex; gap: 4px;">
                  <div style="width: 8px; height: 8px; background: #3b82f6; border-radius: 50%; animation: bounce 1s infinite;"></div>
                  <div style="width: 8px; height: 8px; background: #3b82f6; border-radius: 50%; animation: bounce 1s infinite 0.1s;"></div>
                  <div style="width: 8px; height: 8px; background: #3b82f6; border-radius: 50%; animation: bounce 1s infinite 0.2s;"></div>
                </div>
              </div>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </div>

  <!-- Input -->
  <div style="padding: 16px 24px; border-top: 1px solid #282828; background: #181818;">
    <div style="max-width: 800px; margin: 0 auto;">
      <form onsubmit={(e) => { e.preventDefault(); sendMessage(); }} style="display: flex; gap: 12px;">
        <input
          type="text"
          bind:value={input}
          placeholder="Message..."
          disabled={generating || !selectedModel}
          style="flex: 1; padding: 12px 16px; background: #282828; border: 1px solid #404040; border-radius: 24px; color: white; font-size: 14px;"
        />
        <button
          type="submit"
          disabled={generating || !input.trim() || !selectedModel}
          style="padding: 12px 24px; background: linear-gradient(135deg, #3b82f6, #8b5cf6); border: none; color: white; font-weight: 600; border-radius: 24px; cursor: pointer; font-size: 14px; opacity: {generating || !input.trim() || !selectedModel ? '0.3' : '1'};"
        >
          {generating ? 'Sending...' : 'Send'}
        </button>
      </form>
      {#if !selectedModel && models.length === 0}
        <p style="font-size: 12px; color: #666; margin-top: 8px;">No models installed. Go to Models page to download one.</p>
      {/if}
    </div>
  </div>
</div>

<style>
  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-6px); }
  }

  button:hover:not(:disabled) {
    opacity: 0.9;
  }

  select:hover:not(:disabled) {
    background: #2a2a2a;
  }

  input:hover:not(:disabled) {
    background: #2a2a2a;
  }
</style>
