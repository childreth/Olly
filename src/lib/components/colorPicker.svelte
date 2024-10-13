<script>
  export let color = "#009999";
</script>

<div class="color-picker">
  <input type="color" id="head" name="head" value={color} />
  <label for="head">Theme color</label>
</div>

<style lang="scss">
  .color-picker {
    display: flex;
    flex-direction: row;
    align-items: center;
    font-size: 1rem;
    color: var(--primary);
    gap: 0.5rem;
  }
  input[type="color"] {
    width: 1.5rem;
    height: 1.5rem;
    border: 0;
    padding: 0;
    background-color: transparent;
    -webkit-appearance: none;
    display: flex;
    border-radius:2rem;
   border: 2px solid var(--primary);
  }
  input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
    border-radius:2rem;
  }
  input[type="color"]::-webkit-color-swatch {
    border: none;
    border-radius:2rem;
  }
</style>
