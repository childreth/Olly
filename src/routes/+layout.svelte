<script>
  import "./styles.css";
  import { onMount } from "svelte";

  onMount(() => {
    // Initialize feather icons after DOM is ready
    if (typeof window !== 'undefined' && window.feather) {
      window.feather.replace();
    }
  });
</script>

<svelte:head>
  <script src="/src/lib/feather.min.js"></script>
</svelte:head>

<div id="window">
  <div data-tauri-drag-region class="titlebar">
    <div class="titlebar-button" id="titlebar-minimize"></div>
    <div class="titlebar-button" id="titlebar-maximize"></div>
  </div>

  <main>
    <slot />
  </main>
</div>

<style lang="css">
  #window {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    /* border-radius: 0.5rem; */
    /* overflow: hidden; */
    border: none;
    background-color: rgba(255, 255, 255, 0.5);
  }

  .titlebar {
    height: 28px;
    background-color: transparent;
    user-select: none;
    display: flex;
    width: 100%;
    position: absolute;
    top: 0;
    left: 0;
    z-index: 10;
    justify-content: flex-start;
 
  }
  .titlebar:hover{
  
    
  }
  .titlebar-button {
    display: inline-flex;
    justify-content: center;
    align-items: center;
    width: 24px;
    height: 24px;
  }
  .titlebar-button:hover {
  
  }
</style>
