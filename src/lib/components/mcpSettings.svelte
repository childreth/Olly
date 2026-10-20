<script>
  import Button from "./button.svelte";
  import {
    mcpServers,
    mcpStatus,
    MCP_PRESETS,
    addServer,
    removeServer,
    setServerEnabled,
    connectServer
  } from "$lib/mcp.js";

  let name = "";
  let transport = "http";
  let target = "";
  let formError = "";

  /**
   * Split a command line into words, honouring simple quotes.
   * @param {string} line
   */
  function splitCommand(line) {
    const words = [];
    const re = /"([^"]*)"|'([^']*)'|(\S+)/g;
    let m;
    while ((m = re.exec(line)) !== null) words.push(m[1] ?? m[2] ?? m[3]);
    return words;
  }

  async function handleAdd() {
    formError = "";
    const trimmedName = name.trim();
    const trimmedTarget = target.trim();
    if (!trimmedName || !trimmedTarget) {
      formError = transport === "http" ? "Enter a name and a URL." : "Enter a name and a command.";
      return;
    }
    if (transport === "http" && !/^https?:\/\//i.test(trimmedTarget)) {
      formError = "URL must start with http:// or https://";
      return;
    }
    if (transport === "http") {
      await addServer({ name: trimmedName, transport: "http", url: trimmedTarget });
    } else {
      const [command, ...args] = splitCommand(trimmedTarget);
      await addServer({ name: trimmedName, transport: "stdio", command, args });
    }
    name = "";
    target = "";
  }

  /** @param {{ name: string, transport: string, url: string }} preset */
  function addPreset(preset) {
    addServer({ name: preset.name, transport: "http", url: preset.url });
  }

  /** @param {any} server */
  function describeTarget(server) {
    if (server.transport === "stdio") return [server.command, ...(server.args || [])].join(" ");
    return server.url;
  }

  /**
   * @param {any} status
   * @param {boolean} enabled
   */
  function statusLabel(status, enabled) {
    if (!enabled) return "Off";
    if (!status || status.state === "disconnected") return "Not connected";
    if (status.state === "connecting") return "Connecting…";
    if (status.state === "error") return "Error";
    const count = status.tools?.length ?? 0;
    return `Connected · ${count} tool${count === 1 ? "" : "s"}`;
  }

  $: missingPresets = MCP_PRESETS.filter(
    (p) => !$mcpServers.some((s) => s.transport === "http" && s.url === p.url)
  );
</script>

<div class="mcp-settings">
  <h2 class="text-lg">MCP servers</h2>
  <p class="hint">
    Tools from connected <a href="https://modelcontextprotocol.io" target="_blank" rel="noreferrer">MCP</a> servers are offered to local Ollama models that support tool calling.
  </p>

  {#if $mcpServers.length === 0}
    <p class="empty">No servers yet.</p>
  {/if}

  <div class="server-list">
    {#each $mcpServers as server (server.id)}
      {@const status = $mcpStatus[server.id]}
      <div class="server">
        <div class="server-row">
          <label class="enable">
            <input
              type="checkbox"
              checked={server.enabled}
              on:change={(e) => setServerEnabled(server.id, e.currentTarget.checked)}
              aria-label="Enable {server.name}"
            />
          </label>
          <div class="server-main">
            <div class="server-title">
              <strong>{server.name}</strong>
              <span class="badge {server.enabled ? status?.state || 'disconnected' : 'off'}">{statusLabel(status, server.enabled)}</span>
            </div>
            <code class="target">{describeTarget(server)}</code>
            {#if server.enabled && status?.state === "error"}
              <p class="error">{status.error}</p>
            {/if}
          </div>
          <div class="server-actions">
            <Button
              label="Reconnect"
              icon=""
              type="secondary"
              disabled={!server.enabled || status?.state === "connecting"}
              on:click={() => connectServer(server)}
            />
            <Button label="Remove" icon="" type="secondary" on:click={() => removeServer(server.id)} />
          </div>
        </div>
        {#if server.enabled && status?.state === "connected" && status.tools?.length}
          <details class="tools">
            <summary>Tools</summary>
            <div class="tool-list">
              {#each status.tools as tool}
                <div class="tool"><code>{tool.name}</code>{#if tool.description}<span>&nbsp;— {tool.description.split("\n")[0]}</span>{/if}</div>
              {/each}
            </div>
          </details>
        {/if}
      </div>
    {/each}
  </div>

  <form class="add-form" on:submit|preventDefault={handleAdd}>
    <div class="form-row">
      <div class="form-field name">
        <label for="mcp-name">Name</label>
        <input id="mcp-name" bind:value={name} placeholder="Weft" />
      </div>
      <div class="form-field transport">
        <label for="mcp-transport">Type</label>
        <select id="mcp-transport" bind:value={transport}>
          <option value="http">HTTP</option>
          <option value="stdio">Command (stdio)</option>
        </select>
      </div>
      <div class="form-field target-field">
        <label for="mcp-target">{transport === "http" ? "URL" : "Command"}</label>
        <input
          id="mcp-target"
          bind:value={target}
          placeholder={transport === "http" ? "http://127.0.0.1:4510/mcp" : "npx -y @modelcontextprotocol/server-everything"}
        />
      </div>
      <div class="form-actions">
        <button type="submit" class="add-button">Add</button>
      </div>
    </div>
    {#if formError}
      <p class="error">{formError}</p>
    {/if}
  </form>

  {#if missingPresets.length}
    <div class="presets">
      <span>Suggested:</span>
      {#each missingPresets as preset}
        <Button label="Add {preset.name} ({preset.url})" icon="" type="secondary" on:click={() => addPreset(preset)} />
      {/each}
    </div>
  {/if}
</div>

<style>
  @import "../../routes/forms.css";

  .mcp-settings {
    margin: 0 0 2rem;
  }

  .hint,
  .empty {
    font-size: var(--fontSizeSmall);
    color: var(--secondary);
    margin: 0.25rem 0 1rem;
  }

  .hint a {
    color: inherit;
  }

  .server-list {
    list-style: none;
    padding: 0;
    margin: 0 0 1.5rem;
  }

  .server {
    background-color: var(--surface-2);
    border-bottom: 1px solid var(--tertiary);
    padding: 0.75rem 1rem;
  }

  .server:last-child {
    border-bottom: none;
  }

  .server-row {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .enable {
    padding-top: 0.2rem;
  }

  .server-main {
    flex: 1 1 auto;
    min-width: 0;
  }

  .server-title {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .target {
    display: block;
    font-size: var(--fontSizeXS);
    color: var(--secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .server-actions {
    display: flex;
    gap: 0.5rem;
  }

  .badge {
    padding: 0.125rem 0.375rem;
    border-radius: var(--borderRadiusXS);
    font-size: 0.8rem;
    font-weight: 500;
    background-color: var(--surface-3);
    color: var(--primary);
  }

  .badge.connected {
    background-color: #d4edda;
    color: #155724;
  }

  .badge.error {
    background-color: #f8d7da;
    color: #721c24;
  }

  .error {
    color: #b3261e;
    font-size: var(--fontSizeXS);
    margin: 0.25rem 0 0;
    word-break: break-word;
  }

  .tools {
    margin: 0.5rem 0 0 1.75rem;
    font-size: var(--fontSizeSmall);
  }

  .tools summary {
    cursor: pointer;
    color: var(--secondary);
  }

  .tool-list {
    margin: 0.5rem 0 0;
    padding-left: 1rem;
  }

  .tool {
    margin-bottom: 0.25rem;
  }

  .add-form .form-row {
    flex-wrap: wrap;
  }

  .form-field.name {
    flex: 0 1 8rem;
  }

  .form-field.transport {
    flex: 0 1 10rem;
  }

  .form-field.target-field {
    flex: 1 1 14rem;
  }

  .form-field select {
    width: 100%;
    height: 2rem;
    font-size: var(--fontSizeSmall);
    border: 1px solid var(--secondary);
    border-radius: var(--borderRadiusXS);
    background-color: var(--surface-2);
    color: var(--primary);
    font-family: var(--bodyFamily);
  }

  .add-button {
    font-family: var(--bodyFamily);
    border: 1px solid var(--secondary);
    background: none;
    color: var(--primary);
    min-height: 2rem;
    padding: 0.25rem 1rem;
    font-size: 0.875rem;
    border-radius: var(--borderRadiusXS);
    cursor: pointer;
  }

  .presets {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    margin-top: 1rem;
    font-size: var(--fontSizeSmall);
    flex-wrap: wrap;
  }
</style>
