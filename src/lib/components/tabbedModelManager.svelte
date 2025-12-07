<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Ollama } from "ollama/browser";
  import Button from "./button.svelte";
  import { formatRelativeTime } from '$lib/utils.js';
  
  export let loadModelNames = [];
  export let onModelDeleted = () => {};
  
  let activeTab = "local";
  let loading = false;
  let message = "";
  let messageType = "";
  
  // External API providers
  let externalProviders = [
    { 
      id: "claude", 
      name: "Claude", 
      apiKey: "", 
      displayName: "Claude API",
      placeholder: "Enter Claude API key..."
    },
    { 
      id: "perplexity", 
      name: "Perplexity", 
      apiKey: "", 
      displayName: "Perplexity API",
      placeholder: "Enter Perplexity API key..."
    },
    { 
      id: "openai", 
      name: "OpenAI", 
      apiKey: "", 
      displayName: "OpenAI API",
      placeholder: "Enter OpenAI API key..."
    }
  ];
  
  onMount(async () => {
    await loadExternalProviders();
  });
  
  async function loadExternalProviders() {
    try {
      const providers = await invoke("list_api_key_providers");
      for (const provider of providers) {
        const info = await invoke("get_provider_info", { provider });
        if (info) {
          const providerIndex = externalProviders.findIndex(p => p.id === provider);
          if (providerIndex !== -1) {
            externalProviders[providerIndex].apiKey = "••••••••";
          }
        }
      }
    } catch (error) {
      console.error("Failed to load external providers:", error);
    }
  }
  
  async function deleteLocalModel(model) {
    try {
      const ollama = new Ollama({ host: "http://localhost:11434" });
      await ollama.delete({ model: model });
      onModelDeleted();
      message = `Model ${model} deleted successfully`;
      messageType = "success";
    } catch (error) {
      message = `Failed to delete model ${model}: ${error}`;
      messageType = "error";
    }
  }
  
  async function saveApiKey(provider) {
    const providerObj = externalProviders.find(p => p.id === provider);
    if (!providerObj || !providerObj.apiKey || providerObj.apiKey === "••••••••") return;
    
    loading = true;
    message = "";
    
    try {
      message = `Validating ${providerObj.name} API key...`;
      const isValid = await invoke("validate_api_key", {
        provider: provider,
        apiKey: providerObj.apiKey
      });
      
      if (!isValid) {
        message = `${providerObj.name} API key validation failed - please check your key`;
        messageType = "error";
        loading = false;
        return;
      }
      
      message = `Storing ${providerObj.name} API key...`;
      await invoke("store_api_key", {
        provider: provider,
        displayName: providerObj.displayName,
        apiKey: providerObj.apiKey
      });
      
      message = `${providerObj.name} API key validated and saved successfully`;
      messageType = "success";
      providerObj.apiKey = "••••••••";
      
    } catch (error) {
      console.error(`Error saving ${provider} API key:`, error);
      message = `Failed to save ${providerObj.name} API key: ${error}`;
      messageType = "error";
    } finally {
      loading = false;
    }
  }
  
  async function deleteApiKey(provider) {
    const providerObj = externalProviders.find(p => p.id === provider);
    if (!providerObj) return;
    
    loading = true;
    message = "";
    
    try {
      await invoke("delete_api_key", { provider });
      message = `${providerObj.name} API key deleted successfully`;
      messageType = "success";
      providerObj.apiKey = "";
      
    } catch (error) {
      message = `Failed to delete ${providerObj.name} API key: ${error}`;
      messageType = "error";
    } finally {
      loading = false;
    }
  }
  
  function switchTab(tab) {
    activeTab = tab;
    message = "";
  }
</script>
<h2 class='text-lg'>Manage models</h2>
<div class="tabbed-model-manager">
  <div class="tab-header">
    <button
      class="tab-button {activeTab === 'local' ? 'active' : ''}"
      on:click={() => switchTab('local')}
    >
      Local
    </button>
    <button
      class="tab-button {activeTab === 'external' ? 'active' : ''}"
      on:click={() => switchTab('external')}
    >
      External
    </button>
  </div>
  
  <div class="tab-content">
    {#if message}
      <div class="message {messageType}">{message}</div>
    {/if}
    {#if activeTab === 'local'}
      <div class="local-models">
        <div class="models-table">
          <div class="table-header">
            <span>Name</span>
            <span class="date">Last updated</span>
            <span>Parameter</span>
            <span>Quantization</span>
            <span class="actions">&nbsp;</span>
          </div>
          {#each loadModelNames.filter(model => !model[1].includes('External API')) as model}
            <div class="table-row">
              <span class="">{model[0]}</span>
              <span class="">{formatRelativeTime(model[1])}</span>
              <span>{model[2]}</span>
              <span>{model[3]}</span>
              <span class="actions">
                <Button 
                  label="Delete" 
                  type="secondary" 
                  icon=""
                  on:click={() => deleteLocalModel(model[0])}
                  disabled={loading}
                />
              </span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
    
    {#if activeTab === 'external'}
      <div class="external-models">
        <div class="providers-list">
          {#each externalProviders as provider}
            <div class="provider-card">
              <div class="provider-header">
                <h5>{provider.name}</h5>
                <span class="provider-status {provider.apiKey ? 'connected' : 'disconnected'}">
                  {provider.apiKey ? 'Connected' : 'Not Connected'}
                </span>
              </div>
              <div class="provider-form">
                
                <div class="form-row">
                  <div class="form-field">
                    <label for="{provider.id}-key">API key</label>
                    <input
                      id="{provider.id}-key"
                      type="password"
                      bind:value={provider.apiKey}
                      placeholder={provider.placeholder}
                      disabled={loading}
                    />
                  </div>
                  <div class="form-actions">
                    <Button 
                      label="Save" 
                      icon=""
                      type="secondary"
                      on:click={() => saveApiKey(provider.id)} 
                      disabled={loading || !provider.apiKey || provider.apiKey === "••••••••"}
                    />
                      <Button 
                        label="Delete" 
                        type="secondary" 
                        icon=""
                        on:click={() => deleteApiKey(provider.id)} 
                         disabled={loading || !provider.apiKey }
                      />
                  </div>
                </div>
           
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
   @import "../../routes/forms.css";

  .tabbed-model-manager {
    width: 100%;
  }
  
  .tab-header {
    display: flex;
    border-bottom: 1px solid var(--secondary);
    margin-bottom: 1rem;
  }
  
  .tab-button {
    padding: 0.75rem 1.5rem;
    border: none;
    background: none;
    color: var(--textSecondary);
    font-family: var(--bodyFamily);
    cursor: pointer;
    font-size: 1rem;
    border-bottom: 2px solid transparent;

  }
  
  .tab-button:hover {
    color: var(--primary);
    background-color: var(--surfaceHover);
  }
  
  .tab-button.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
    /* background-color: var(--surfaceActive); */
  }
  
  .tab-content {
    min-height: 300px;
  }
  
  .message {
    padding: 0.5rem .75rem;
    border-radius: var(--borderRadiusXS);
    margin-bottom: 1rem;
    font-weight: 500;
    font-size: var(--fontSizeSmall);
    border: 1px solid var(--tertiary);
    position: sticky;
    top: .5rem;
    z-index: 2;
  }
  
  .message.success {
    background-color: #d4edda;
    color: #155724;
    border: 1px solid #c3e6cb;
  }
  
  .message.error {
    background-color: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
  }
  
  .local-models h4,
  .external-models h4 {
    margin-bottom: 1rem;
    color: var(--primary);
  }
  
  .models-table {
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  
  .table-header,
  .table-row {
    display: grid;
    grid-template-columns: 2fr 1.5fr 1fr 1fr 0.5fr;
    gap:0;
    align-items: center;
    padding: 0.75rem 0;
  }
  
  .table-header {
    font-weight: 600;
    color: var(--textSecondary);
    position: sticky;
    top: 0;
    z-index: 1;
    padding:  0.5rem;
    border-radius: var(--borderRadiusXS);
    background-color: var(--surface-2);
    font-size: var(--fontSizeSmall);
  }
  
  .table-row {

    font-size: var(--fontSizeMedium);
    padding:  0.75rem;
    border-bottom: 1px solid var(--tertiary);

  }
  
  .table-row:hover {
    background-color: var(--surfaceHover);
  }
  
  
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  
  .providers-list {
    display: flex;
    flex-direction: column;
    gap: 0;
    margin-bottom:3rem;
  }
  
  .provider-card {
    border-bottom: 1px solid var(--tertiary);
    padding: 1rem 1rem 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    background-color: var(--surface-2);
  }
  .provider-card:last-child {
    border-bottom: none;
  }
  
  .provider-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: .5rem;
  }
  
  .provider-header h5 {
    margin: 0;
    color: var(--primary);
    font-size: 1.1rem;
  }
  
  .provider-status {
    padding: 0.125rem 0.375rem;
    border-radius: var(--borderRadiusXS);
    font-size: 0.8rem;
    font-weight: 500;
  }
  
  .provider-status.connected {
    background-color: #d4edda;
    color: #155724;
  }
  
  .provider-status.disconnected {
    background-color: #f8d7da;
    color: #721c24;
  }
  
  .provider-form {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    width: 100%;
  }

  .provider-form .form-row {
    flex: 1 1 auto;
    min-width: 50%;
  }

  
  
  .form-actions {
    display: flex;
    gap: 0.5rem;
  }
</style>