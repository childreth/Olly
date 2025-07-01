<script>
  import { createEventDispatcher } from 'svelte';

  export let options = [];
  export let value = '';
  export let placeholder = 'Search models...';
  export let displayKey = 'name';
  export let valueKey = 'id';

  const dispatch = createEventDispatcher();

  let searchTerm = '';
  let isOpen = false;
  let filteredOptions = [];
  let selectedIndex = -1;
  let dropdownElement;
  let inputElement;

  $: {
    if (searchTerm === '') {
      filteredOptions = options;
    } else {
      filteredOptions = options.filter(option =>
        option[displayKey].toLowerCase().includes(searchTerm.toLowerCase()) ||
        (option.description && option.description.toLowerCase().includes(searchTerm.toLowerCase()))
      );
    }
    selectedIndex = -1;
  }

  $: selectedOption = options.find(option => option[valueKey] === value);
  $: displayValue = selectedOption ? selectedOption[displayKey] : '';

  function handleInputFocus() {
    isOpen = true;
    searchTerm = '';
  }

  function handleInputBlur(event) {
    // Delay closing to allow click events on options
    setTimeout(() => {
      if (!dropdownElement?.contains(document.activeElement)) {
        isOpen = false;
        searchTerm = '';
      }
    }, 150);
  }

  function selectOption(option) {
    value = option[valueKey];
    searchTerm = '';
    isOpen = false;
    inputElement?.blur();
    dispatch('change', { value: option[valueKey], option });
  }

  function handleKeyDown(event) {
    if (!isOpen) {
      if (event.key === 'Enter' || event.key === 'ArrowDown') {
        event.preventDefault();
        isOpen = true;
        selectedIndex = 0;
      }
      return;
    }

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, filteredOptions.length - 1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, -1);
        break;
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && filteredOptions[selectedIndex]) {
          selectOption(filteredOptions[selectedIndex]);
        }
        break;
      case 'Escape':
        event.preventDefault();
        isOpen = false;
        searchTerm = '';
        inputElement?.blur();
        break;
    }
  }
</script>

<div class="searchable-select">
  <input
    bind:this={inputElement}
    bind:value={searchTerm}
    on:focus={handleInputFocus}
    on:blur={handleInputBlur}
    on:keydown={handleKeyDown}
    placeholder={isOpen ? placeholder : displayValue || placeholder}
    class="search-input"
    autocomplete="off"
  />
  
  {#if isOpen && filteredOptions.length > 0}
    <div bind:this={dropdownElement} class="dropdown">
      {#each filteredOptions as option, index}
        <div
          class="option {index === selectedIndex ? 'highlighted' : ''}"
          on:click={() => selectOption(option)}
          on:keydown
        >
          <div class="option-main">
            <span class="option-name">{option[displayKey]}</span>
            {#if option.provider}
              <span class="option-provider">{option.provider}</span>
            {/if}
          </div>
          {#if option.description}
            <div class="option-description">{option.description}</div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .searchable-select {
    position: relative;
    width: 100%;
    max-width: 320px;
  }

  .search-input {
    width: 100%;
    padding: 0 2.75rem 0 1.5rem;
    border: 0;
    min-height: 2.5rem;
    line-height: 2.5rem;
    font-size: 1rem;
    color: var(--primary);
    border: 1px solid var(--secondary);
    border-radius: 2rem;
    font-family: var(--bodyFamily);
    background-color: transparent;
    background-image: url("$lib/images/keyboard_arrow_down.svg");
    background-repeat: no-repeat;
    background-position: calc(100% - 0.75rem) center;
    background-size: 24px;
    box-sizing: border-box;
  }
  .search-input::placeholder {
    color: var(--primary);
  }

  .search-input:hover {
    cursor: pointer;
    box-shadow: 2px 2px 0 0 var(--secondary);
    transform: translate(-1px, -1px);
    background-color: var(--surface);
    transition: border-width 0.1s ease-in-out, box-shadow 0.1s ease-in-out;
  }

  .search-input:focus {
    outline: 2px solid var(--secondary);
    background-image: none;
  }

  .dropdown {
    position: absolute;
    top: 100%;
    /* left: 0; */
    right: 0;
    width:100%;
    min-width: 240px;
    background: var(--surface);
    border: 1px solid var(--secondary);
    border-radius: 1rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    z-index: 1000;
    max-height: 300px;
    overflow-y: auto;
    margin-top: 0.25rem;
  }

  .option {
    padding: 0.75rem 1rem;
    cursor: pointer;
    border-bottom: 1px solid var(--tertiary);
  }

  .option:last-child {
    border-bottom: none;
  }

  .option:hover,
  .option.highlighted {
    background-color: var(--transparent24);
  }

  .option-main {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .option-name {
    font-weight: 500;
    font-size: 1rem;
    line-height: 1.25rem;
    color: var(--primary);
  }

  .option-provider {
    font-size: 0.625rem;
    color: var(--secondary);
    background: var(--tertiary);
    padding: 0.125rem 0.5rem;
    border-radius: 1rem;
    text-transform: capitalize;
  }

  .option-description {
    font-size: 0.75rem;
    color: var(--secondary);
    line-height: 1.3;
  }
</style>