<script>
  // logic goes here
  export let label = "default";
  export let type = "default";
  export let icon;
  export let disabled = false;
  
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();
  
  function handleClick() {
    if (!disabled) {
      dispatch('click');
    }
  }
</script>

<!-- markup (zero or more items) goes here -->
<div id="buttonWrap" class="{type}">
  <button on:click={handleClick} {disabled}>{label}</button>
  <!-- {#if icon}
    <img src="$lib/images/{icon}.svg" alt="{icon} icon">
  {/if} -->
</div>

<style>
  #buttonWrap {
    position: relative;
    background-color:var(--secondary);
    display: inline-block;
    padding:1px;
    border-radius:var(--borderRadiusXXS);

  }

  button {
    background-color: var(--buttonbg);
    font-family: var(--bodyFamily);
    padding: .25rem 1rem;
    color: var(--surface);
    font-size: 1rem;
    font-weight: 400;
    min-height: 30px;
    border-radius: .375rem;
    display: flex;
    gap: 4px;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border: none;
    transition: all cubic-bezier(0.67, -0.04, 0.31, 1.04) 0.4s;
  }

  #buttonWrap.secondary button {
    background-color: var(--surface);
    color: var(--primary);
  }
  
  @property --angle {
    syntax: "<angle>";
    initial-value: 0deg;
    inherits: false;
  }

  /* .button-border:after {
    content: "";
    position: absolute;
    height: 100%;
    width: 100%;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background-color: var(--secondary);
    padding: 1px;
    transition: background 0.2s ease-in-out;
  } */

  #buttonWrap.stopBtn:after {
    background: conic-gradient(
      from var(--angle) at 50% 50%,
      var(--primary) 0% 30%,
      var(--surface) 70% 100%
    );
    animation: spin 1s linear infinite;
    transition: background 0.3s ease-in-out;
  }
  @keyframes spin {
    from {
      --angle: 0deg;
    }
    to {
      --angle: 360deg;
    }
  }

  

  #buttonWrap:has(button:focus-visible) {
    box-shadow: 0 0 0 3px var(--surface), 0 0 0 6px var(--primary);
    border-radius: 12rem;
  }
  button:focus-visible {
    outline: none;
  }

  #buttonWrap:hover button:not(:disabled) {
    background-position: 0.75rem 40%;
  }
  
  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }



 
  
</style>
