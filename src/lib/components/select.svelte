<script>
  export let options = [
    {'label':'Outfit', 'value':'Outfit'},
    {'label':'Comic Sans', 'value':'Comic Sans'},
    {'label':'Times New Roman', 'value':'Times New Roman'},
    {'label':'Monospace', 'value':'Monospace'},
    {'label':'System Font', 'value':'System Font'}
  ];
  export let value = options[0].value; // Set default value to the first option
  export let label = '';
  export let id = '';
  export let name = '';
  export let small = false;

  function handleChange(event) {
    value = event.target.value;
  }
</script>

<div class="input">
  {#if label}
    <label for={id}>{label}</label>
  {/if}
  <select
    {id}
    {name}
    bind:value
    class={small ? 'small' : ''}
  >
    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
</div>

<style>

label {
    font-size: .875rem;
}
select.small {
  min-height: 1.75rem;
  line-height: 1.75rem;
  padding: 0 2.75rem .125rem 1rem;
}
select {
    color: var(--primary);
}
 
</style>
