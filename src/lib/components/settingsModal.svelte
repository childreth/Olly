<script>
  import { invoke } from "@tauri-apps/api/core";
  import Button from "./button.svelte";
  
  export let isOpen = false;
  
  let apiKeys = {
    claude: "",
    perplexity: "",
    openai: ""
  };
  
  let loading = false;
  let message = "";
  let messageType = ""; // "success" or "error"
  
  // Load existing API keys on component mount
  async function loadApiKeys() {
    try {
      const providers = await invoke("list_api_key_providers");
      for (const provider of providers) {
        const info = await invoke("get_provider_info", { provider });
        if (info) {
          // Don't load the actual key for security, just mark as set
          apiKeys[provider] = "••••••••";
        }
      }
    } catch (error) {
      console.error("Failed to load API keys:", error);
    }
  }
  
  async function saveApiKey(provider) {
    if (!apiKeys[provider] || apiKeys[provider] === "••••••••") return;
    
    loading = true;
    message = "";
    
    try {
      // First validate the API key
      message = `Validating ${provider} API key...`;
      const isValid = await invoke("validate_api_key", {
        provider,
        apiKey: apiKeys[provider]
      });
      
      if (!isValid) {
        message = `${provider} API key validation failed - please check your key`;
        messageType = "error";
        loading = false;
        return;
      }
      
      // If valid, store it
      message = `Storing ${provider} API key...`;
      console.log(`About to store API key for ${provider}`);
      
      await invoke("store_api_key", {
        provider,
        displayName: `${provider.charAt(0).toUpperCase() + provider.slice(1)} API`,
        apiKey: apiKeys[provider]
      });
      
      console.log(`Successfully called store_api_key for ${provider}`);
      
      // Verify it was saved by checking the fresh storage
      const providers = await invoke("list_api_key_providers");
      console.log(`After saving ${provider}, fresh storage shows:`, providers);
      
      message = `${provider} API key validated and saved successfully`;
      messageType = "success";
      apiKeys[provider] = "••••••••"; // Hide the key after saving
      
    } catch (error) {
      console.error(`Error saving ${provider} API key:`, error);
      message = `Failed to save ${provider} API key: ${error}`;
      messageType = "error";
    } finally {
      loading = false;
    }
  }
  
  async function deleteApiKey(provider) {
    loading = true;
    message = "";
    
    try {
      await invoke("delete_api_key", { provider });
      message = `${provider} API key deleted successfully`;
      messageType = "success";
      apiKeys[provider] = "";
      
    } catch (error) {
      message = `Failed to delete ${provider} API key: ${error}`;
      messageType = "error";
    } finally {
      loading = false;
    }
  }
  
  async function migrateKeys() {
    loading = true;
    message = "";
    
    try {
      await invoke("migrate_api_keys");
      message = "API keys migrated successfully from config file";
      messageType = "success";
      await loadApiKeys(); // Reload keys after migration
      
    } catch (error) {
      message = `Migration failed: ${error}`;
      messageType = "error";
    } finally {
      loading = false;
    }
  }
  
  async function migrateClaudeKey() {
    loading = true;
    message = "";
    
    try {
      const result = await invoke("migrate_claude_key");
      message = result;
      messageType = result.includes("✅") ? "success" : "info";
      await loadApiKeys(); // Reload keys after migration
      
    } catch (error) {
      message = `Claude migration failed: ${error}`;
      messageType = "error";
    } finally {
      loading = false;
    }
  }
  
  $: if (isOpen) {
    loadApiKeys();
  }
  
  function closeModal() {
    isOpen = false;
    message = "";
  }
  
  async function testStoreLoad() {
    try {
      const result = await invoke("test_store_load");
      console.log("Fresh Storage test:", result);
      message = result;
      messageType = result.includes("PASSED") ? "success" : "error";
    } catch (error) {
      console.error("Fresh Storage test failed:", error);
      message = `Fresh Storage test error: ${error}`;
      messageType = "error";
    }
  }
  
  async function debugKeys() {
    try {
      const result = await invoke("debug_api_keys");
      console.log("Debug Keys:", result);
      message = result;
      messageType = "info";
    } catch (error) {
      console.error("Debug Keys failed:", error);
      message = `Debug Keys error: ${error}`;
      messageType = "error";
    }
  }
  
  async function testStoreDirect() {
    try {
      console.log("Testing direct store with debug version...");
      const storeResult = await invoke("store_api_key_debug", {
        provider: "test_debug",
        displayName: "Test Debug",
        apiKey: "test_key_abc123"
      });
      console.log("Debug store result:", storeResult);
      
      // Try to retrieve it with debug version
      const retrieveResult = await invoke("get_api_key_debug", {
        provider: "test_debug"
      });
      console.log("Debug retrieve result:", retrieveResult);
      
      message = `Store: ${storeResult.substring(0, 50)}... | Retrieve: ${retrieveResult.substring(0, 50)}...`;
      messageType = retrieveResult.includes("FOUND") ? "success" : "error";
      
    } catch (error) {
      console.error("Direct store test failed:", error);
      message = `Direct store error: ${error}`;
      messageType = "error";
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-overlay" on:click={closeModal}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h2>API Key Settings</h2>
        <button class="close-btn" aria-label="Close settings" on:click={closeModal}></button>
      </div>
      
      <div class="modal-body">
        {#if message}
          <div class="message {messageType}">{message}</div>
        {/if}
        
        <div class="migration-section">
          <h3>Migration</h3>
          <p>Migrate API keys to new secure file storage:</p>
          <Button label="Migrate Claude Key" on:click={migrateClaudeKey} />
          <Button label="Legacy Migration" type="secondary" on:click={migrateKeys} />
        </div>
        
        <div class="api-keys-section">
          <h3>API Keys</h3>
          
          <!-- Claude API Key -->
          <div class="key-group">
            <label for="claude-key">Claude API Key:</label>
            <div class="key-input-group">
              <input
                id="claude-key"
                type="password"
                bind:value={apiKeys.claude}
                placeholder="Enter Claude API key..."
                disabled={loading}
              />
              <div class="key-actions">
                <Button label="Save" on:click={() => saveApiKey('claude')} disabled={loading} />
                {#if apiKeys.claude === "••••••••"}
                  <Button label="Delete" type="secondary" on:click={() => deleteApiKey('claude')} disabled={loading} />
                {/if}
              </div>
            </div>
          </div>
          
          <!-- Perplexity API Key -->
          <div class="key-group">
            <label for="perplexity-key">Perplexity API Key:</label>
            <div class="key-input-group">
              <input
                id="perplexity-key"
                type="password"
                bind:value={apiKeys.perplexity}
                placeholder="Enter Perplexity API key..."
                disabled={loading}
              />
              <div class="key-actions">
                <Button label="Save" on:click={() => saveApiKey('perplexity')} disabled={loading} />
                {#if apiKeys.perplexity === "••••••••"}
                  <Button label="Delete" type="secondary" on:click={() => deleteApiKey('perplexity')} disabled={loading} />
                {/if}
              </div>
            </div>
          </div>
          
          <!-- OpenAI API Key -->
          <div class="key-group">
            <label for="openai-key">OpenAI API Key:</label>
            <div class="key-input-group">
              <input
                id="openai-key"
                type="password"
                bind:value={apiKeys.openai}
                placeholder="Enter OpenAI API key..."
                disabled={loading}
              />
              <div class="key-actions">
                <Button label="Save" on:click={() => saveApiKey('openai')} disabled={loading} />
                {#if apiKeys.openai === "••••••••"}
                  <Button label="Delete" type="secondary" on:click={() => deleteApiKey('openai')} disabled={loading} />
                {/if}
              </div>
            </div>
          </div>
          
          <!-- Debug Section -->
          <div class="key-group">
            <Button label="Test Fresh Storage" type="secondary" on:click={testStoreLoad} disabled={loading} />
            <Button label="Debug Keys" type="secondary" on:click={debugKeys} disabled={loading} />
            <Button label="Test Store Direct" type="secondary" on:click={testStoreDirect} disabled={loading} />
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  
  .modal-content {
    background-color: var(--surface-2);
    border-radius: var(--borderRadiusS);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
    width: 90%;
    max-width: 600px;
    max-height: 80vh;
    overflow-y: auto;
  }
  
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--secondary);
  }
  
  .modal-header h2 {
    margin: 0;
    color: var(--primary);
    font-size: 1.5rem;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 2rem;
    color: var(--primary);
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }
  
  .close-btn:hover {
    color: var(--accent);
  }
  
  .modal-body {
    padding: 1.5rem;
  }
  
  .message {
    padding: 0.75rem;
    border-radius: var(--borderRadiusXS);
    margin-bottom: 1rem;
    font-weight: 500;
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
  
  .migration-section, .api-keys-section {
    margin-bottom: 2rem;
  }
  
  .migration-section h3, .api-keys-section h3 {
    color: var(--primary);
    margin-bottom: 1rem;
    font-size: 1.25rem;
  }
  
  .migration-section p {
    color: var(--textSecondary);
    margin-bottom: 1rem;
  }
  
  .key-group {
    margin-bottom: 1.5rem;
  }
  
  .key-group label {
    display: block;
    color: var(--primary);
    font-weight: 500;
    margin-bottom: 0.5rem;
  }
  
  .key-input-group {
    display: flex;
    gap: 1rem;
    align-items: center;
  }
  
  .key-input-group input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid var(--secondary);
    border-radius: var(--borderRadiusXS);
    background-color: var(--surface-2);
    color: var(--primary);
    font-family: var(--bodyFamily);
  }
  
  .key-input-group input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(var(--accent), 0.2);
  }
  
  .key-input-group input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .key-actions {
    display: flex;
    gap: 0.5rem;
  }
</style>