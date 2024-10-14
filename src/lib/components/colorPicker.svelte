<script>
  export let color = "#009999";

  function handleColorChange(event) {
    color = event.target.value;
    console.log("handleColorChange");
    // Convert hex to HSL and extract hue
    const hexToHSL = (hex) => {
      // Remove the hash if it's there
      hex = hex.replace(/^#/, "");

      // Convert hex to RGB
      let r = parseInt(hex.slice(0, 2), 16) / 255;
      let g = parseInt(hex.slice(2, 4), 16) / 255;
      let b = parseInt(hex.slice(4, 6), 16) / 255;

      // Find greatest and smallest channel values
      let cmin = Math.min(r, g, b),
        cmax = Math.max(r, g, b),
        delta = cmax - cmin,
        h = 0;

      // Calculate hue
      if (delta === 0) h = 0;
      else if (cmax === r) h = ((g - b) / delta) % 6;
      else if (cmax === g) h = (b - r) / delta + 2;
      else h = (r - g) / delta + 4;

      h = Math.round(h * 60);
      if (h < 0) h += 360;

      return h;
    };

    const hue = hexToHSL(color);
    document.documentElement.style.setProperty("--hue", hue);
    let style = getComputedStyle(document.body);
    //localStorage.setItem('themeColor', color);
  }
</script>

<div class="color-picker">
  <input
    type="color"
    id="head"
    on:change={handleColorChange}
    name="head"
    value={color}
  />
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
    border-radius: 2rem;
    border: 2px solid var(--primary);
  }
  input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
    border-radius: 2rem;
  }
  input[type="color"]::-webkit-color-swatch {
    border: none;
    border-radius: 2rem;
  }
</style>
