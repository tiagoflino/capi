<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, tick } from 'svelte';
  import { selectedModel, currentSessionId, isGenerating, inferenceMetrics, triggerNewChat } from '$lib/stores/app';
  import { marked } from 'marked';

  // Local state for chat
  let messages = $state<any[]>([]);
  let input = $state('');
  let messagesEnd = $state<HTMLDivElement | null>(null);
  let chatScroller = $state<HTMLDivElement | null>(null);
  let autoScroll = true;

  // Configure marked for safety and tables
  marked.setOptions({
    gfm: true,
    breaks: true
  });

  // Track the most recent session we've loaded to avoid redundant reloads
  let lastLoadedSessionId = $state<string | null>(null);

  // React to Session ID changes from Sidebar
  $effect(() => {
    if ($currentSessionId && $currentSessionId !== lastLoadedSessionId && !$isGenerating) {
      loadMessages($currentSessionId);
    } else if (!$currentSessionId) {
      messages = [];
      lastLoadedSessionId = null;
    }
  });

  // React to "New Chat" trigger
  $effect(() => {
    const _ = $triggerNewChat;
    if (!$currentSessionId) {
      messages = [];
      lastLoadedSessionId = null;
    }
  });

  async function loadMessages(sessionId: string) {
    try {
      lastLoadedSessionId = sessionId;
      const historyDocs = await invoke('get_chat_messages', { sessionId });
      messages = historyDocs.map((m: any) => ({
        role: m.role,
        content: m.content
      }));
      await tick();
      forceScrollToBottom();
    } catch (e) {
      console.error('Failed to load history:', e);
    }
  }

  function handleScroll() {
    if (!chatScroller) return;
    const { scrollTop, scrollHeight, clientHeight } = chatScroller;
    // If we're within 100px of bottom, stick to bottom
    autoScroll = scrollHeight - scrollTop - clientHeight < 100;
  }

  function forceScrollToBottom() {
    if (chatScroller) {
      chatScroller.scrollTop = chatScroller.scrollHeight;
    }
  }

  $effect(() => {
    // Whenever messages change, if autoScroll is enabled, scroll to bottom
    if (messages.length && autoScroll) {
      tick().then(forceScrollToBottom);
    }
  });

  async function sendMessage() {
    if (!input.trim() || !$selectedModel || $isGenerating) return;

    const userMessage = input;
    input = '';
    
    // Add user message
    messages = [...messages, { role: 'user', content: userMessage }];
    $isGenerating = true;
    autoScroll = true; // Re-enable autoscroll on sent message
    await tick();
    forceScrollToBottom();

    const assistantMsgIndex = messages.length;
    messages = [...messages, { role: 'assistant', content: '' }];

    let unlistenTokens: (() => void) | undefined;
    let unlistenMetrics: (() => void) | undefined;

    try {
      unlistenTokens = await listen('chat-token', (event: any) => {
        messages[assistantMsgIndex].content += event.payload.token;
        messages = [...messages];
      });

      unlistenMetrics = await listen('chat-metrics', (event: any) => {
        $inferenceMetrics = event.payload;
      });

      let targetSessionId = $currentSessionId;
      if (!targetSessionId) {
         targetSessionId = await invoke('create_chat_session', { 
            modelId: $selectedModel,
            title: userMessage.slice(0, 30) // Use first chunk of prompt as title
         }) as string;
         // Key Fix: Set these values so the $effect doesn't trigger a reload 
         // which would clear our current streaming messages array.
         lastLoadedSessionId = targetSessionId;
         $currentSessionId = targetSessionId;
      }

      const metrics: any = await invoke('chat_direct', {
        modelId: $selectedModel,
        prompt: userMessage,
        sessionId: targetSessionId
      });

      if (messages[assistantMsgIndex]) {
        messages[assistantMsgIndex].metrics = {
          tokens_per_second: metrics.tokens_per_second,
          completion_tokens: metrics.num_output_tokens,
        };
        messages = [...messages];
      }
      
    } catch (e: any) {
      console.error('Chat error:', e);
      if (messages[assistantMsgIndex]) {
        messages[assistantMsgIndex].content += `\n\n**Error:** ${e}`;
        messages = [...messages];
      }
    } finally {
      if (unlistenTokens) unlistenTokens();
      if (unlistenMetrics) unlistenMetrics();
      $isGenerating = false;
    }
  }
</script>

<div class="chat-page">
  <div class="messages-viewport" bind:this={chatScroller} onscroll={handleScroll}>
    <div class="messages-container">
      {#if messages.length === 0}
        <div class="empty-state">
           <div class="logo-hero">
             <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
               <circle cx="12" cy="12" r="10" stroke="#b87333" stroke-dasharray="4 4"/>
               <path d="M12 11V13M12 17H12.01" stroke="#b87333" stroke-linecap="round" stroke-width="2"/>
               <circle cx="12" cy="12" r="4" fill="#fae8d1"/>
             </svg>
           </div>
           <h1>Capybara</h1>
           <p>Intelligent Local Inference</p>
           <div class="quick-tips">
             <div class="tip">Support for Markdown & Tables</div>
             <div class="tip">Low-latency GPU acceleration</div>
             <div class="tip">Private & Offline</div>
           </div>
        </div>
      {:else}
        {#each messages as msg}
          <div class="message-row {msg.role}">
            <div class="bubble">
              <div class="role-header">
                 <span class="role-name">{msg.role === 'user' ? 'You' : 'Capybara'}</span>
                 {#if msg.metrics}
                    <span class="m-stat">{msg.metrics.tokens_per_second.toFixed(1)} t/s</span>
                 {/if}
              </div>
              <div class="markdown-body">
                {@html marked.parse(msg.content)}
              </div>
            </div>
          </div>
        {/each}
        {#if $isGenerating}
           <div class="message-row assistant">
             <div class="bubble typing-bubble">
               <div class="typing-indicator">
                 <span></span><span></span><span></span>
               </div>
             </div>
           </div>
        {/if}
      {/if}
      <div bind:this={messagesEnd} style="height: 1px; visibility: hidden;"></div>
    </div>
  </div>

  <div class="input-area">
    <div class="input-wrap">
      <form onsubmit={(e) => { e.preventDefault(); sendMessage(); }} class="chat-form">
        <textarea 
          bind:value={input} 
          placeholder="Ask anything..." 
          disabled={$isGenerating || !$selectedModel}
          onkeydown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              sendMessage();
            }
          }}
          rows="1"
        ></textarea>
        <button type="submit" disabled={!input.trim() || $isGenerating || !$selectedModel} class="send-btn">
          {#if $isGenerating}
            <div class="stop-dot"></div>
          {:else}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
              <path d="M12 19V5M5 12l7-7 7 7" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          {/if}
        </button>
      </form>
    </div>
  </div>
</div>

<style>
  .chat-page {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: #fdfcfb;
    position: relative;
    overflow: hidden;
  }

  .messages-viewport {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    scroll-behavior: auto; /* Fast scrolling during generation */
  }

  .messages-container {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    padding-bottom: 140px;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .empty-state { margin-top: 15vh; text-align: center; }
  .logo-hero { width: 64px; height: 64px; margin: 0 auto 24px; color: #b87333; }
  .empty-state h1 { font-size: 32px; font-weight: 800; color: #3d3b38; letter-spacing: -0.04em; }
  .empty-state p { color: #8c8984; font-size: 17px; margin-bottom: 40px; }

  .quick-tips { display: flex; justify-content: center; gap: 12px; flex-wrap: wrap; }
  .tip { padding: 8px 16px; background: white; border: 1px solid rgba(0,0,0,0.05); border-radius: 12px; font-size: 13px; color: #8c8984; font-weight: 500; }

  .message-row { display: flex; width: 100%; }
  .message-row.user { justify-content: flex-end; }
  
  .bubble {
    max-width: 85%;
    padding: 16px 20px;
    border-radius: 20px;
    font-size: 15px;
    line-height: 1.6;
    background: white;
    border: 1px solid rgba(0,0,0,0.05);
    color: #3d3b38;
    box-shadow: 0 4px 12px rgba(0,0,0,0.02);
  }

  .message-row.user .bubble {
    background: #f7f3ef;
    border-bottom-right-radius: 4px;
    border-color: rgba(184, 115, 51, 0.1);
  }
  
  .message-row.assistant .bubble {
    background: transparent;
    border: none;
    box-shadow: none;
    padding-left: 0;
    width: 100%;
    max-width: 100%;
  }

  .role-header { 
    display: flex; 
    align-items: center; 
    justify-content: space-between;
    margin-bottom: 10px;
  }

  .role-name { font-size: 12px; font-weight: 700; color: #b87333; text-transform: uppercase; letter-spacing: 0.05em; }
  .m-stat { font-size: 10px; font-weight: 700; color: #8c8984; font-family: monospace; }

  /* Markdown Styling */
  .markdown-body :global(p) { margin-bottom: 12px; }
  .markdown-body :global(p:last-child) { margin-bottom: 0; }
  .markdown-body :global(pre) { background: #f4f2f0; padding: 16px; border-radius: 12px; overflow-x: auto; margin: 12px 0; border: 1px solid rgba(0,0,0,0.05); font-family: 'Fira Code', monospace; font-size: 13px; }
  .markdown-body :global(code) { font-family: 'Fira Code', monospace; background: rgba(0,0,0,0.05); padding: 2px 4px; border-radius: 4px; font-size: 0.9em; }
  .markdown-body :global(table) { width: 100%; border-collapse: collapse; margin: 16px 0; font-size: 14px; }
  .markdown-body :global(th), .markdown-body :global(td) { border: 1px solid rgba(0,0,0,0.1); padding: 8px 12px; text-align: left; }
  .markdown-body :global(th) { background: #fbf9f6; font-weight: 700; }
  .markdown-body :global(ul), .markdown-body :global(ol) { padding-left: 20px; margin-bottom: 12px; }
  .markdown-body :global(li) { margin-bottom: 4px; }
  .markdown-body :global(blockquote) { border-left: 4px solid #fae8d1; padding-left: 16px; margin: 12px 0; color: #8c8984; font-style: italic; }
  .markdown-body :global(hr) { border: 0; border-top: 1px solid rgba(0,0,0,0.06); margin: 16px 0; }
  .markdown-body :global(input[type="checkbox"]) { margin-right: 8px; vertical-align: middle; }

  .input-area {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 32px 24px;
    background: linear-gradient(to top, #fdfcfb 70%, transparent);
    display: flex;
    justify-content: center;
  }

  .input-wrap {
    width: 100%;
    max-width: 800px;
    background: white;
    border-radius: 24px;
    box-shadow: 0 12px 48px -12px rgba(0,0,0,0.12);
    border: 1px solid rgba(0,0,0,0.06);
    padding: 10px 14px;
  }

  .chat-form { display: flex; align-items: flex-end; gap: 12px; }
  
  textarea {
    flex: 1;
    border: none;
    outline: none;
    padding: 10px 4px;
    font-size: 16px;
    background: transparent;
    color: #3d3b38;
    resize: none;
    max-height: 200px;
    font-family: inherit;
  }
  textarea::placeholder { color: #a8a29e; }
  
  .send-btn {
    width: 40px;
    height: 40px;
    background: #3d3b38;
    color: white;
    border: none;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    flex-shrink: 0;
    margin-bottom: 2px;
  }
  .send-btn:hover:not(:disabled) { transform: translateY(-1px); background: #000; }
  .send-btn:disabled { opacity: 0.15; cursor: default; }
  
  .stop-dot { width: 12px; height: 12px; background: white; border-radius: 2px; }

  .typing-bubble { padding: 12px 16px !important; }
  .typing-indicator { display: flex; gap: 4px; align-items: center; height: 12px; }
  .typing-indicator span { 
    width: 6px; height: 6px; background: #b87333; border-radius: 50%; opacity: 0.4;
    animation: pulse 1.4s infinite ease-in-out;
  }
  .typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
  .typing-indicator span:nth-child(3) { animation-delay: 0.4s; }
  
  @keyframes pulse { 0%, 100% { transform: scale(0.8); opacity: 0.4; } 50% { transform: scale(1.2); opacity: 0.8; } }
</style>
