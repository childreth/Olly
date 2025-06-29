<script>
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  
  export let message = '';
  export let type = 'info'; // 'info', 'success', 'error'
  export let visible = false;
  export let duration = 3000;
  
  let timeoutId;
  
  $: if (visible && message) {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => {
      visible = false;
    }, duration);
  }
  
  function close() {
    visible = false;
    if (timeoutId) clearTimeout(timeoutId);
  }
</script>

{#if visible && message}
  <div class="toast toast-{type}" transition:fly="{{ y: -50, duration: 300 }}">
    <div class="toast-content">
      <span class="toast-message">{message}</span>
      <button class="toast-close" on:click={close}>&times;</button>
    </div>
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    top: 20px;
    right: 20px;
    z-index: 9999;
    max-width: 400px;
    border-radius: var(--borderRadiusS);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    overflow: hidden;
  }
  
  .toast-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    gap: 12px;
  }
  
  .toast-message {
    flex: 1;
    color: white;
    font-size: 0.9rem;
    line-height: 1.4;
  }
  
  .toast-close {
    background: none;
    border: none;
    color: white;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0;
    opacity: 0.7;
    transition: opacity 0.2s;
  }
  
  .toast-close:hover {
    opacity: 1;
  }
  
  .toast-info {
    background-color: #3b82f6;
  }
  
  .toast-success {
    background-color: #10b981;
  }
  
  .toast-error {
    background-color: #ef4444;
  }
</style>