<script>
  // logic goes here
  export let label = "default";
  export let type = "default";
  export let icon = "";
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
<div id="buttonWrap">
  <button class="{type}" on:click={handleClick} {disabled}>
  {#if type !== 'icon-only'}
    {label}
  {/if}
    {#if icon}
    <img src="/images/{icon}.svg" alt="{icon} icon">
  {/if}
  </button>
 
</div>

<style scoped>
  #buttonWrap {
    position: relative;
  }

  button {
    background-color: var(--buttonbg);
    background-image: none;
    font-family: var(--bodyFamily);
    padding: .25rem 1rem;
    color: var(--primary);
    font-size: 1rem;
    font-weight: 500;
    border-radius: var(--borderRadiusXS);
    display: flex;
    min-height: 2rem;
    gap: 4px;
    border: none;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    /* transition: all cubic-bezier(0.67, -0.04, 0.31, 1.04) 0.4s; */
    box-sizing: border-box;
}
  
.secondary {
  font-family: var(--bodyFamily);
  border: 1px solid var(--secondary);
  min-height:2rem;
  font-size: 0.875rem;
  color: var(--primary);
  background-color: transparent;
}

.secondary:hover {
  background-color: var(--tertiary);
  background-image: none;
  color: var(--primary);
  box-shadow: none;
}

.icon-only {
  background-color: transparent;
  max-width: 2rem;
  max-height: 2rem;

}


.link {
  background-color: transparent;
  border: none;
  font-size: var(--fontSizeMedium);
  color: var(--primary);
  padding: .5rem .25rem;
}

.link:hover {
  box-shadow: none;
  background-color: transparent;
  text-decoration: underline;
}

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    color: var(--primary);
    background-color: var(--tertiary);
  }

  
  
</style>
