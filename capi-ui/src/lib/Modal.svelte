<script lang="ts">
  let { isOpen, onClose, title, children }: { isOpen: boolean; onClose: () => void; title: string; children: any } = $props();
</script>

{#if isOpen}
<div
    class="modal-backdrop"
    onclick={onClose}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <div
      class="modal-content"
      onclick={(e) => e.stopPropagation()}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div style="padding: 20px 24px; border-bottom: 1px solid #282828; display: flex; align-items: center; justify-content: between;">
        <h2 style="font-size: 20px; font-weight: 600; flex: 1;">{title}</h2>
        <button
          onclick={onClose}
          style="width: 32px; height: 32px; border-radius: 8px; background: transparent; border: none; color: #888; cursor: pointer; font-size: 20px;"
        >
          ×
        </button>
      </div>

      <!-- Content -->
      <div style="flex: 1; overflow-y: auto; padding: 24px;">
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
  }

  .modal-content {
    background: #181818;
    border: 1px solid #282828;
    border-radius: 16px;
    max-width: 800px;
    width: 90%;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  button:hover {
    background: #282828 !important;
    color: white !important;
  }
</style>
