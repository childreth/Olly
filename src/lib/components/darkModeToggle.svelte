<script>
  export let checked = true;
  export let cta = true;
  export let id = '';
  import { onMount } from 'svelte';

  onMount(() => {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      checked = savedTheme === 'dark';
      document.body.setAttribute('data-theme', savedTheme);
    }
  });
  
  $: mode = checked ? 'on' : 'off';

  function handleChange() {
    checked = !checked;

    const body = document.body;
    const currentTheme = body.getAttribute('data-theme');
  
  if (!checked) {
    body.setAttribute('data-theme', 'light');
    localStorage.setItem('theme', 'light');
  } else {
    body.setAttribute('data-theme', 'dark');
    localStorage.setItem('theme', 'dark');
  }
  console.log('localStorage',localStorage.getItem('theme'))
  }
</script>

<div class="toggle-container">
  <label for={id}>
    <input
      type="checkbox"
      {id}
      bind:checked
      on:click={handleChange}
    />
    <span class="toggle-switch"></span>
    {#if cta}
      <span class="toggle-label">Dark mode ({mode})</span>
    {/if}
  </label>
</div>

<style>
  .toggle-container {
    display: inline-block;
  }

  label {
    display: flex;
    align-items: center;
    cursor: pointer;
    font-size:1rem;
  }

  input[type="checkbox"] {
    display: none;
  }

  .toggle-switch {
    position: relative;
    width: 36px;
    height: 16px;
    background-color: var(--surface-2);
    border: 2px solid var(--secondary);
    border-radius: 12px;
    transition: background-color 0.3s;
    margin-inline-end: 0.125rem;
  }

  .toggle-switch::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 8px;
    height:8px;
    
    border: 2px solid var(--secondary);
    border-radius: 50%;
    transition: transform 0.3s cubic-bezier(.26,0,.08,1);
  }

  input[type="checkbox"]:checked + .toggle-switch {
    background-color: var(--surface-2);
  }

  input[type="checkbox"]:checked + .toggle-switch::after {
    transform: translateX(20px);
    background-color: var(--primary);
    border-color: var(--surface-2);
  }

  .toggle-label {
    margin-left: 8px;
    font-family: var(--bodyFamily);
    color: var(--text);
  }
</style>
