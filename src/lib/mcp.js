/**
 * MCP (Model Context Protocol) client support for Ollama chats.
 *
 * Transport lives in Rust (src-tauri/src/mcp.rs) and is reached via invoke():
 * native requests carry no webview Origin and need no CORS, which local servers
 * like Weft (http://127.0.0.1:4510/mcp) require. This module owns:
 *  - server config (persisted in localStorage, like other Olly settings)
 *  - connection status + discovered tools (Svelte stores)
 *  - converting MCP tools to Ollama function tools (namespaced `server__tool`)
 *  - executing tool calls and formatting results for the model and the UI
 *
 * The pure helpers (no Tauri, no DOM) are exported for testing.
 */
import { invoke } from "@tauri-apps/api/core";
import { writable, get } from "svelte/store";

const STORAGE_KEY = "mcpServers";

/** Characters of a tool result fed back to the model (protects small context windows). */
export const MAX_RESULT_CHARS_FOR_MODEL = 16000;
/** Characters of a tool result shown in the chat. */
export const MAX_RESULT_CHARS_FOR_UI = 4000;
/** Max depth when inlining `$ref`s in tool schemas (recursive schemas get cut off here). */
const MAX_SCHEMA_DEPTH = 6;

/** Suggested servers shown in Settings. Nothing is added automatically. */
export const MCP_PRESETS = [
  { name: "Weft", transport: "http", url: "http://127.0.0.1:4510/mcp" }
];

/**
 * @typedef {{ id: string, name: string, transport: 'http' | 'stdio', url?: string, command?: string, args?: string[], enabled: boolean }} McpServer
 * @typedef {{ name: string, description?: string, inputSchema?: any }} McpTool
 * @typedef {{ state: 'disconnected' | 'connecting' | 'connected' | 'error', error?: string, lastAttempt?: number, tools?: McpTool[], instructions?: string, serverInfo?: any }} McpStatus
 */

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/**
 * Turn a server name into a tool-name prefix: lowercase, [a-z0-9_], <= 20 chars.
 * @param {string} name
 */
export function slugify(name) {
  const slug = String(name || "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 20);
  return slug || "mcp";
}

/**
 * Resolve a local JSON pointer like `#/definitions/foo` against the root schema.
 * @param {any} root
 * @param {string} ref
 */
function resolveRef(root, ref) {
  if (typeof ref !== "string" || !ref.startsWith("#/")) return null;
  let node = root;
  for (const part of ref.slice(2).split("/")) {
    const key = part.replace(/~1/g, "/").replace(/~0/g, "~");
    if (node == null || typeof node !== "object") return null;
    node = node[key];
  }
  return node ?? null;
}

/** Keys Ollama's tool templates and models can use; everything else is dropped. */
const KEEP_KEYS = ["type", "description", "enum", "default", "required", "minimum", "maximum", "minItems", "maxItems", "format", "const"];

/**
 * Simplify one JSON Schema node: inline `$ref`s, flatten simple `anyOf`/`oneOf`
 * into a type list, keep descriptive keywords, recurse into properties/items.
 * @param {any} schema
 * @param {any} root
 * @param {number} depth
 * @returns {any}
 */
function simplifySchema(schema, root, depth) {
  if (schema === true || schema == null || typeof schema !== "object") return {};
  if (schema.$ref) {
    const target = depth < MAX_SCHEMA_DEPTH ? resolveRef(root, schema.$ref) : null;
    if (!target) return { type: "object", description: schema.description || "Nested object (same shape as its parent)" };
    const merged = { ...target, ...schema };
    delete merged.$ref;
    return simplifySchema(merged, root, depth + 1);
  }

  /** @type {any} */
  const out = {};
  for (const key of KEEP_KEYS) {
    if (schema[key] !== undefined) out[key] = schema[key];
  }
  if (out.description !== undefined && typeof out.description !== "string") delete out.description;
  if (out.required !== undefined && !Array.isArray(out.required)) delete out.required;

  const union = schema.anyOf || schema.oneOf;
  if (!out.type && Array.isArray(union)) {
    const members = union.map((m) => simplifySchema(m, root, depth + 1));
    const types = [...new Set(members.flatMap((m) => (Array.isArray(m.type) ? m.type : m.type ? [m.type] : [])))];
    if (types.length === 1) out.type = types[0];
    else if (types.length > 1) out.type = types;
    // Keep the richest object/array member's structure so nested fields survive.
    const structured = members.find((m) => m.properties || m.items);
    if (structured) {
      if (structured.properties) out.properties = structured.properties;
      if (structured.required && !out.required) out.required = structured.required;
      if (structured.items) out.items = structured.items;
    }
    const enums = members.flatMap((m) => m.enum || (m.const !== undefined ? [m.const] : []));
    if (!out.enum && enums.length) out.enum = enums;
  }

  if (schema.properties && typeof schema.properties === "object") {
    out.properties = {};
    for (const [key, value] of Object.entries(schema.properties)) {
      out.properties[key] = simplifySchema(value, root, depth + 1);
    }
    if (!out.type) out.type = "object";
  }
  if (schema.items) {
    out.items = Array.isArray(schema.items)
      ? simplifySchema(schema.items[0], root, depth + 1)
      : simplifySchema(schema.items, root, depth + 1);
    if (!out.type) out.type = "array";
  }
  if (out.const !== undefined && !out.enum) {
    out.enum = [out.const];
  }
  delete out.const;
  return out;
}

/**
 * Convert an MCP tool `inputSchema` (JSON Schema) to Ollama function `parameters`.
 * Always returns `{ type: 'object', properties, required? }`.
 * @param {any} inputSchema
 */
export function toOllamaParameters(inputSchema) {
  const schema = inputSchema && typeof inputSchema === "object" ? inputSchema : {};
  const simplified = simplifySchema(schema, schema, 0);
  /** @type {any} */
  const params = {
    type: "object",
    properties: simplified.properties || {},
    required: Array.isArray(simplified.required) ? simplified.required : []
  };
  return params;
}

/**
 * Build Ollama tool definitions for every connected server's tools, plus an
 * index from namespaced name back to server + original tool name.
 * @param {McpServer[]} servers
 * @param {Record<string, McpStatus>} statuses
 */
export function buildToolIndex(servers, statuses) {
  /** @type {any[]} */
  const ollamaTools = [];
  /** @type {Map<string, { serverId: string, serverName: string, toolName: string }>} */
  const index = new Map();
  const usedPrefixes = new Set();

  for (const server of servers) {
    const status = statuses[server.id];
    if (!server.enabled || !status || status.state !== "connected") continue;

    let prefix = slugify(server.name);
    for (let n = 2; usedPrefixes.has(prefix); n++) prefix = `${slugify(server.name).slice(0, 17)}_${n}`;
    usedPrefixes.add(prefix);

    for (const tool of status.tools || []) {
      if (!tool || typeof tool.name !== "string") continue;
      const name = `${prefix}__${tool.name}`.replace(/[^a-zA-Z0-9_-]/g, "_").slice(0, 64);
      if (index.has(name)) continue;
      index.set(name, { serverId: server.id, serverName: server.name, toolName: tool.name });
      ollamaTools.push({
        type: "function",
        function: {
          name,
          description: `[${server.name}] ${tool.description || tool.name}`,
          parameters: toOllamaParameters(tool.inputSchema)
        }
      });
    }
  }
  return { ollamaTools, index };
}

/**
 * System-prompt addendum with each connected server's instructions.
 * Returns "" when no server is connected, so the prompt is unchanged.
 * @param {McpServer[]} servers
 * @param {Record<string, McpStatus>} statuses
 */
export function buildInstructions(servers, statuses) {
  const sections = [];
  const usedPrefixes = new Set();
  for (const server of servers) {
    const status = statuses[server.id];
    if (!server.enabled || !status || status.state !== "connected") continue;
    let prefix = slugify(server.name);
    for (let n = 2; usedPrefixes.has(prefix); n++) prefix = `${slugify(server.name).slice(0, 17)}_${n}`;
    usedPrefixes.add(prefix);
    const body = typeof status.instructions === "string" && status.instructions.trim()
      ? status.instructions.trim()
      : "(No instructions provided.)";
    sections.push(`## ${server.name} (tools prefixed \`${prefix}__\`)\n${body}`);
  }
  if (sections.length === 0) return "";
  return `\n\n# Connected tool servers (MCP)\nYou can call tools from these servers. Call a tool when it helps answer the user; pass arguments as a JSON object matching the tool's parameters. If a tool returns an error, read it, fix your arguments, and try again or explain the problem.\n\n${sections.join("\n\n")}`;
}

/**
 * Normalize tool-call arguments from the model. Small models sometimes send a
 * JSON string, null, or garbage instead of an object.
 * @param {any} args
 * @returns {{ ok: true, value: Record<string, any> } | { ok: false, error: string }}
 */
export function normalizeArguments(args) {
  if (args == null || args === "") return { ok: true, value: {} };
  if (typeof args === "string") {
    try {
      const parsed = JSON.parse(args);
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) return { ok: true, value: parsed };
      return { ok: false, error: "Tool arguments must be a JSON object." };
    } catch (e) {
      return { ok: false, error: `Tool arguments are not valid JSON: ${/** @type {Error} */ (e).message}` };
    }
  }
  if (typeof args === "object" && !Array.isArray(args)) return { ok: true, value: args };
  return { ok: false, error: "Tool arguments must be a JSON object." };
}

/**
 * Flatten an MCP CallToolResult into text.
 * @param {any} result
 * @returns {{ text: string, isError: boolean }}
 */
export function formatToolResult(result) {
  if (!result || typeof result !== "object") return { text: String(result ?? ""), isError: false };
  const parts = [];
  for (const item of Array.isArray(result.content) ? result.content : []) {
    if (!item || typeof item !== "object") continue;
    if (item.type === "text") parts.push(item.text ?? "");
    else if (item.type === "image") parts.push(`[image: ${item.mimeType || "unknown type"}]`);
    else if (item.type === "audio") parts.push(`[audio: ${item.mimeType || "unknown type"}]`);
    else if (item.type === "resource") parts.push(item.resource?.text ?? `[resource: ${item.resource?.uri || "unknown"}]`);
    else if (item.type === "resource_link") parts.push(`[resource: ${item.uri || item.name || "unknown"}]`);
    else parts.push(JSON.stringify(item));
  }
  if (parts.length === 0 && result.structuredContent !== undefined) {
    parts.push(JSON.stringify(result.structuredContent, null, 2));
  }
  return { text: parts.join("\n"), isError: result.isError === true };
}

/**
 * @param {string} text
 * @param {number} max
 */
export function truncate(text, max) {
  if (text.length <= max) return text;
  return `${text.slice(0, max)}\n… [truncated ${text.length - max} characters]`;
}

// ---------------------------------------------------------------------------
// Stores + persistence
// ---------------------------------------------------------------------------

function loadServers() {
  try {
    const raw = typeof localStorage !== "undefined" ? localStorage.getItem(STORAGE_KEY) : null;
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.filter((s) => s && s.id && s.name) : [];
  } catch {
    return [];
  }
}

/** @type {import('svelte/store').Writable<McpServer[]>} */
export const mcpServers = writable(loadServers());
/** @type {import('svelte/store').Writable<Record<string, McpStatus>>} */
export const mcpStatus = writable({});

mcpServers.subscribe((servers) => {
  try {
    if (typeof localStorage !== "undefined") localStorage.setItem(STORAGE_KEY, JSON.stringify(servers));
  } catch {
    // Storage unavailable: settings last for this session only.
  }
});

/**
 * @param {string} id
 * @param {McpStatus} status
 */
function setStatus(id, status) {
  mcpStatus.update((all) => ({ ...all, [id]: status }));
}

/** @param {McpServer} server */
function transportConfig(server) {
  if (server.transport === "stdio") {
    return { transport: "stdio", command: server.command || "", args: server.args || [], env: {} };
  }
  return { transport: "http", url: server.url || "" };
}

/**
 * Connect a server and record its tools/instructions. Never throws.
 * @param {McpServer} server
 */
export async function connectServer(server) {
  setStatus(server.id, { state: "connecting" });
  try {
    const info = await invoke("mcp_connect", { serverId: server.id, config: transportConfig(server) });
    setStatus(server.id, {
      state: "connected",
      tools: Array.isArray(info?.tools) ? info.tools : [],
      instructions: typeof info?.instructions === "string" ? info.instructions : "",
      serverInfo: info?.serverInfo ?? null
    });
  } catch (error) {
    setStatus(server.id, { state: "error", error: String(error), lastAttempt: Date.now() });
  }
}

/** @param {string} id */
export async function disconnectServer(id) {
  try {
    await invoke("mcp_disconnect", { serverId: id });
  } catch (error) {
    console.warn("MCP disconnect failed:", error);
  }
  setStatus(id, { state: "disconnected" });
}

/** Minimum gap between automatic reconnect attempts for a failed server. */
const RECONNECT_INTERVAL_MS = 15000;

/**
 * Before a chat turn: retry enabled servers that are down (e.g. Weft started
 * after Olly). Failed connects to a local port are near-instant.
 */
export async function reconnectStaleServers() {
  const statuses = get(mcpStatus);
  const now = Date.now();
  const stale = get(mcpServers).filter((s) => {
    if (!s.enabled) return false;
    const st = statuses[s.id];
    if (!st) return true;
    if (st.state !== "error" && st.state !== "disconnected") return false;
    return now - (st.lastAttempt || 0) > RECONNECT_INTERVAL_MS;
  });
  await Promise.all(stale.map((s) => connectServer(s)));
}

/** Connect every enabled server (app start). */
export async function connectEnabledServers() {
  await Promise.all(get(mcpServers).filter((s) => s.enabled).map((s) => connectServer(s)));
}

/**
 * Add a server and connect it.
 * @param {{ name: string, transport?: 'http' | 'stdio', url?: string, command?: string, args?: string[] }} spec
 */
export async function addServer(spec) {
  /** @type {McpServer} */
  const server = {
    id: `mcp-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`,
    name: spec.name.trim(),
    transport: spec.transport === "stdio" ? "stdio" : "http",
    url: spec.url?.trim(),
    command: spec.command?.trim(),
    args: spec.args || [],
    enabled: true
  };
  mcpServers.update((all) => [...all, server]);
  await connectServer(server);
  return server;
}

/** @param {string} id */
export async function removeServer(id) {
  mcpServers.update((all) => all.filter((s) => s.id !== id));
  await disconnectServer(id);
  mcpStatus.update((all) => {
    const next = { ...all };
    delete next[id];
    return next;
  });
}

/**
 * @param {string} id
 * @param {boolean} enabled
 */
export async function setServerEnabled(id, enabled) {
  mcpServers.update((all) => all.map((s) => (s.id === id ? { ...s, enabled } : s)));
  const server = get(mcpServers).find((s) => s.id === id);
  if (!server) return;
  if (enabled) await connectServer(server);
  else await disconnectServer(id);
}

// ---------------------------------------------------------------------------
// Chat integration
// ---------------------------------------------------------------------------

/** Tools + index + instructions for the currently connected servers. */
export function getActiveMcpTools() {
  const servers = get(mcpServers);
  const statuses = get(mcpStatus);
  const { ollamaTools, index } = buildToolIndex(servers, statuses);
  return { ollamaTools, index, instructions: buildInstructions(servers, statuses) };
}

/**
 * Execute one tool call from the model. Never throws: every failure becomes an
 * error result that is shown in chat and fed back to the model.
 * @param {Map<string, { serverId: string, serverName: string, toolName: string }>} index
 * @param {string} name - namespaced tool name from the model
 * @param {any} rawArgs
 * @returns {Promise<{ text: string, isError: boolean, args: any, serverName: string | null }>}
 */
export async function callMcpTool(index, name, rawArgs) {
  const target = index.get(name);
  if (!target) {
    const available = [...index.keys()].join(", ");
    return { text: `Unknown tool \`${name}\`. Available tools: ${available}`, isError: true, args: rawArgs, serverName: null };
  }
  const args = normalizeArguments(rawArgs);
  if (!args.ok) {
    return { text: args.error, isError: true, args: rawArgs, serverName: target.serverName };
  }
  try {
    const result = await invoke("mcp_call_tool", {
      serverId: target.serverId,
      name: target.toolName,
      arguments: args.value
    });
    const formatted = formatToolResult(result);
    return { ...formatted, args: args.value, serverName: target.serverName };
  } catch (error) {
    const message = String(error);
    // Mark the server as errored when the transport itself failed.
    if (/Cannot reach|not connected|exited|timed out|HTTP 5/i.test(message)) {
      mcpStatus.update((all) => ({ ...all, [target.serverId]: { ...(all[target.serverId] || {}), state: "error", error: message, lastAttempt: Date.now() } }));
    }
    return { text: message, isError: true, args: args.value, serverName: target.serverName };
  }
}
