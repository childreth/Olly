<script>
  let isChecked = false;

  function handleToggle() {
    isChecked = !isChecked;
  }
</script>

<label class="toggle">
  <input type="checkbox" bind:checked={isChecked} on:change={handleToggle}>
  <span class="slider"></span>
</label>

<style>
  .toggle {
    position: relative;
    display: inline-block;
    width: 60px;
    height: 34px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--tertiary);
    transition: .4s;
    border-radius: 34px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 26px;
    width: 26px;
    left: 4px;
    bottom: 4px;
    background-color: var(--surface);
    transition: .4s;
    border-radius: 50%;
  }

  input:checked + .slider {
    background-color: var(--secondary);
  }

  input:checked + .slider:before {
    transform: translateX(26px);
  }

  input:focus-visible + .slider {
    box-shadow: 0 0 1px var(--secondary);
  }
</style>
