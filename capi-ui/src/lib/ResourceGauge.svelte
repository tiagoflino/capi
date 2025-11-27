<script lang="ts">
  let { label, used, total, unit = 'GB' }: { label: string; used: number; total: number; unit?: string } = $props();

  const percentage = (used / total) * 100;
  const color = percentage > 80 ? '#ef4444' : percentage > 60 ? '#eab308' : '#22c55e';
</script>

<div style="margin-bottom: 8px;">
  <div style="display: flex; justify-content: space-between; font-size: 10px; color: #888; margin-bottom: 4px;">
    <span>{label}</span>
    <span>{used.toFixed(1)}/{total.toFixed(1)} {unit}</span>
  </div>
  <div style="height: 4px; background: #1a1a1a; border-radius: 2px; overflow: hidden;">
    <div style="width: {percentage}%; height: 100%; background: {color}; transition: all 0.3s;"></div>
  </div>
</div>
