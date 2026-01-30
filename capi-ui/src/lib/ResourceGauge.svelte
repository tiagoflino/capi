<script lang="ts">
  let { label, used, total, unit = 'GB' }: { label: string; used: number; total: number; unit?: string } = $props();

  const percentage = (used / total) * 100;
  // Using more sophisticated 'Capybara' colors for the progress bars
  const color = percentage > 85 ? '#ff453a' : percentage > 70 ? '#ff9f0a' : '#b87333';
</script>

<div class="gauge-container">
  <div class="gauge-meta">
    <span class="gauge-label">{label}</span>
    <span class="gauge-value">{used.toFixed(1)}/{total.toFixed(1)} {unit}</span>
  </div>
  <div class="gauge-track">
    <div 
      class="gauge-fill" 
      style="width: {Math.min(100, percentage)}%; background: {color};"
    ></div>
  </div>
</div>

<style>
  .gauge-container {
    margin-bottom: 10px;
  }
  .gauge-meta {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    font-weight: 600;
    color: #8c8984;
    margin-bottom: 4px;
    font-family: monospace;
  }
  .gauge-label {
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .gauge-track {
    height: 6px;
    background: rgba(0, 0, 0, 0.05);
    border-radius: 3px;
    overflow: hidden;
  }
  .gauge-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
  }
</style>
