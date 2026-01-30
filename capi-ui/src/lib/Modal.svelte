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
      class="modal-content glass"
      onclick={(e) => e.stopPropagation()}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="modal-header">
        <h2 class="modal-title">{title}</h2>
        <button
          onclick={onClose}
          class="close-btn"
        >
          ×
        </button>
      </div>

      <!-- Content -->
      <div class="modal-body">
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(61, 59, 56, 0.4);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .modal-content {
    background: #fcfaf7;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 24px;
    max-width: 600px;
    width: 90%;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.1);
    animation: modalPop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  @keyframes modalPop {
    from { opacity: 0; transform: scale(0.95) translateY(10px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  .modal-header {
    padding: 20px 24px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .modal-title {
    font-size: 18px;
    font-weight: 800;
    letter-spacing: -0.02em;
    color: #3d3b38;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.05);
    border: none;
    color: #8c8984;
    cursor: pointer;
    font-size: 24px;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: rgba(0, 0, 0, 0.1);
    color: #3d3b38;
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .glass {
    background: rgba(252, 250, 247, 0.9);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
</style>
