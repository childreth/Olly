<script>
  import { tick } from 'svelte';

  // logic goes here
  export let label = "default";
  export let elID;

  // Re-render feather icons when elID changes
  $: if (elID) {
    tick().then(() => {
      // @ts-ignore - feather is loaded globally
      if (typeof feather !== 'undefined') {
        // @ts-ignore
        feather.replace();
      }
    });
  }
</script>

<!-- markup (zero or more items) goes here -->
<div id="buttonWrap" class="button-border button-anime {elID}">
  <button on:click id={elID}>
    {#key elID}
      {#if elID === 'stopBtn'}
        <i data-feather="stop-circle"></i>
      {:else}
        <i data-feather="arrow-up"></i>
      {/if}
    {/key}
    {label}</button>
</div>

<style scoped>
  #buttonWrap {
    position: relative;
    z-index: 0;
  }
  @property --angle {
    syntax: "<angle>";
    initial-value: 0deg;
    inherits: false;
  }

  .button-border:after {
    content: "";
    position: absolute;
    height: 100%;
    width: 100%;
    top: 50%;
    left: 50%;
    border-radius: 12rem;
    transform: translate(-50%, -50%);
    background-color: var(--primary);
    z-index: -1;
    padding: 2px;
    transition: background 0.3s ease-in-out;
  }

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

  button {
    /* background: var(--buttonbg) url("$lib/images/arrow-up.svg") no-repeat
      0.75rem 50%; */
    /* box-shadow: inset 0 -2px 8px 0 var(--buttonShadow); */
    background: var(--buttonbg);
    background-size: 24px auto;
    font-family: var(--bodyFamily);
    padding: 0rem 1.25rem 0 1rem;
    min-height: 2.5rem;
    min-width: 8.25ch;
    /* min-width: 10ch; */
    color: var(--surface);
    font-size: 1.25rem;
    font-weight: 700;
    line-height: 2.75rem;
    border-radius: 12rem;
    display: flex;
    gap: .25rem;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border: none;
    transition: transform 0.2s ease-in-out;
  }
  #buttonWrap:has(button:focus-visible) {
    box-shadow: 0 0 0 3px var(--surface), 0 0 0 6px var(--primary);
    border-radius: 12rem;
  }
  button:focus-visible {
    outline: none;
  }
  .feather {
    stroke-width: 2px;
    width: 20px;
    height: 20px;
 
  }
  #buttonWrap button :global(svg) {
    transition: transform 0.2s ease-in-out;
  } 

  #buttonWrap:hover button :global(svg) {
    /* transform: translateY(-2px) translateX(0px); */
  }

  #buttonWrap button#stopBtn {
    background: var(--surface);
    background-image: none;

    /* background: var(--surface) url("$lib/images/stop.svg") no-repeat 0.75rem 50%; */
    color: var(--primary);
  }
  #buttonWrap:hover button#stopBtn {
    background-position: 0.75rem 50%;
  }

</style>
