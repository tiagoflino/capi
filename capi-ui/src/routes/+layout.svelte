<script lang="ts">
  import '../app.css';
  import AppSidebar from '$lib/components/AppSidebar.svelte';
  
  let { children } = $props();
</script>

<div class="layout-root">
  <AppSidebar />
  
  <main class="main-viewport">
    {@render children()}
  </main>
</div>

<style>
  .layout-root {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: #ffffff;
  }

  .main-viewport {
    flex: 1;
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
