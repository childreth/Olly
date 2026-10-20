//! Minimal MCP (Model Context Protocol) client.
//!
//! Lives on the Rust side so requests come from the native process instead of
//! the webview: no CORS, no webview `Origin` header (reqwest sends none), and no
//! `@tauri-apps/plugin-http` URL scope to maintain. Local servers that reject
//! foreign origins (e.g. Weft on 127.0.0.1:4510) accept these requests.
//!
//! Transports:
//! - Streamable HTTP (`{"transport":"http","url":...}`): JSON-RPC over POST,
//!   responses as `application/json` or `text/event-stream`, `Mcp-Session-Id`
//!   tracked, expired sessions re-initialized once.
//! - stdio (`{"transport":"stdio","command":...,"args":[...]}`): spawned child
//!   process speaking newline-delimited JSON-RPC.
//!
//! Only the client features Olly needs are implemented: initialize, tools/list,
//! tools/call. No sampling/roots/elicitation capabilities are advertised.

use futures_util::StreamExt;
use log::{info, warn};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{oneshot, Mutex};

const PROTOCOL_VERSION: &str = "2025-06-18";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const CALL_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "transport", rename_all = "lowercase")]
pub enum ServerConfig {
    Http {
        url: String,
    },
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
}

#[derive(Debug)]
enum RpcError {
    /// HTTP 404 on a request carrying a session id: the client must re-initialize.
    SessionExpired,
    Other(String),
}

impl From<String> for RpcError {
    fn from(s: String) -> Self {
        RpcError::Other(s)
    }
}

impl RpcError {
    fn into_string(self) -> String {
        match self {
            RpcError::SessionExpired => "MCP session expired".to_string(),
            RpcError::Other(s) => s,
        }
    }
}

/// Turn a JSON-RPC response object into its `result` or an error string.
fn unwrap_response(msg: Value) -> Result<Value, RpcError> {
    if let Some(err) = msg.get("error") {
        let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
        let message = err.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
        return Err(RpcError::Other(format!("MCP error {}: {}", code, message)));
    }
    Ok(msg.get("result").cloned().unwrap_or(Value::Null))
}

fn matches_id(msg: &Value, id: u64) -> bool {
    msg.get("id").and_then(|v| v.as_u64()) == Some(id)
        && (msg.get("result").is_some() || msg.get("error").is_some())
}

// ---------------------------------------------------------------------------
// Streamable HTTP
// ---------------------------------------------------------------------------

struct HttpTransport {
    client: reqwest::Client,
    url: String,
    session_id: Mutex<Option<String>>,
    protocol_version: Mutex<Option<String>>,
}

impl HttpTransport {
    fn new(url: String) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            client,
            url,
            session_id: Mutex::new(None),
            protocol_version: Mutex::new(None),
        })
    }

    async fn post(&self, body: &Value) -> Result<reqwest::Response, RpcError> {
        let mut req = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(body);
        let session = self.session_id.lock().await.clone();
        if let Some(sid) = &session {
            req = req.header("Mcp-Session-Id", sid);
        }
        if let Some(pv) = self.protocol_version.lock().await.clone() {
            req = req.header("MCP-Protocol-Version", pv);
        }
        let res = req.send().await.map_err(|e| {
            if e.is_connect() {
                RpcError::Other(format!("Cannot reach MCP server at {} (is it running?)", self.url))
            } else {
                RpcError::Other(format!("MCP request failed: {}", e))
            }
        })?;
        if res.status() == reqwest::StatusCode::NOT_FOUND && session.is_some() {
            return Err(RpcError::SessionExpired);
        }
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            let snippet: String = text.chars().take(300).collect();
            return Err(RpcError::Other(format!("MCP server returned HTTP {}: {}", status, snippet)));
        }
        if let Some(sid) = res.headers().get("mcp-session-id").and_then(|v| v.to_str().ok()) {
            *self.session_id.lock().await = Some(sid.to_string());
        }
        Ok(res)
    }

    async fn request(&self, id: u64, method: &str, params: Value) -> Result<Value, RpcError> {
        let body = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let res = self.post(&body).await?;
        let content_type = res
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_lowercase();

        if content_type.starts_with("text/event-stream") {
            return read_sse_response(res, id).await;
        }

        let msg: Value = res
            .json()
            .await
            .map_err(|e| RpcError::Other(format!("Invalid JSON from MCP server: {}", e)))?;
        // A server may batch; pick our response out of an array.
        let msg = match msg {
            Value::Array(items) => items
                .into_iter()
                .find(|m| matches_id(m, id))
                .ok_or_else(|| RpcError::Other("MCP response missing from batch".into()))?,
            other => other,
        };
        unwrap_response(msg)
    }

    async fn notify(&self, method: &str, params: Value) -> Result<(), RpcError> {
        let body = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        self.post(&body).await.map(|_| ())
    }

    async fn close(&self) {
        let session = self.session_id.lock().await.take();
        if let Some(sid) = session {
            // Best effort: tell the server it can drop the session.
            let _ = self
                .client
                .delete(&self.url)
                .header("Mcp-Session-Id", sid)
                .timeout(Duration::from_secs(2))
                .send()
                .await;
        }
    }
}

/// Read an SSE response body until the JSON-RPC response with `id` arrives.
async fn read_sse_response(res: reqwest::Response, id: u64) -> Result<Value, RpcError> {
    let mut stream = res.bytes_stream();
    let mut buffer = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| RpcError::Other(format!("MCP stream error: {}", e)))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk).replace("\r\n", "\n"));
        while let Some(pos) = buffer.find("\n\n") {
            let event: String = buffer.drain(..pos + 2).collect();
            let data: Vec<&str> = event
                .lines()
                .filter_map(|l| l.strip_prefix("data:"))
                .map(|l| l.strip_prefix(' ').unwrap_or(l))
                .collect();
            if data.is_empty() {
                continue;
            }
            if let Ok(msg) = serde_json::from_str::<Value>(&data.join("\n")) {
                if matches_id(&msg, id) {
                    return unwrap_response(msg);
                }
                // Notifications (progress, logging) are ignored.
            }
        }
    }
    Err(RpcError::Other("MCP stream ended without a response".into()))
}

// ---------------------------------------------------------------------------
// stdio
// ---------------------------------------------------------------------------

type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>;

struct StdioTransport {
    stdin: Mutex<tokio::process::ChildStdin>,
    child: Mutex<tokio::process::Child>,
    pending: Pending,
}

impl StdioTransport {
    fn spawn(command: &str, args: &[String], env: &HashMap<String, String>) -> Result<Self, String> {
        // GUI apps on macOS start with a minimal PATH; add the usual tool locations
        // so commands like `npx`/`uvx` resolve.
        let base_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("/opt/homebrew/bin:/usr/local/bin:{}", base_path);
        let mut child = tokio::process::Command::new(command)
            .args(args)
            .envs(env)
            .env("PATH", path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to start `{}`: {}", command, e))?;

        let stdin = child.stdin.take().ok_or("No stdin for MCP process")?;
        let stdout = child.stdout.take().ok_or("No stdout for MCP process")?;
        let stderr = child.stderr.take();
        let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

        let reader_pending = pending.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let Ok(msg) = serde_json::from_str::<Value>(&line) else { continue };
                if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
                    if msg.get("result").is_some() || msg.get("error").is_some() {
                        if let Some(tx) = reader_pending.lock().await.remove(&id) {
                            let _ = tx.send(msg);
                        }
                    }
                }
            }
            // Process exited: fail everything still waiting.
            reader_pending.lock().await.clear();
        });
        if let Some(stderr) = stderr {
            let name = command.to_string();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    info!("[mcp:{}] {}", name, line);
                }
            });
        }

        Ok(Self { stdin: Mutex::new(stdin), child: Mutex::new(child), pending })
    }

    async fn write(&self, msg: &Value) -> Result<(), RpcError> {
        let mut line = serde_json::to_string(msg).map_err(|e| e.to_string())?;
        line.push('\n');
        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| RpcError::Other(format!("MCP process not writable: {}", e)))?;
        stdin.flush().await.map_err(|e| RpcError::Other(e.to_string()))
    }

    async fn request(&self, id: u64, method: &str, params: Value) -> Result<Value, RpcError> {
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        if let Err(e) = self
            .write(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
            .await
        {
            self.pending.lock().await.remove(&id);
            return Err(e);
        }
        let msg = rx
            .await
            .map_err(|_| RpcError::Other("MCP process exited before responding".into()))?;
        unwrap_response(msg)
    }

    async fn notify(&self, method: &str, params: Value) -> Result<(), RpcError> {
        self.write(&json!({ "jsonrpc": "2.0", "method": method, "params": params })).await
    }

    async fn close(&self) {
        let _ = self.child.lock().await.kill().await;
    }
}

// ---------------------------------------------------------------------------
// Connection
// ---------------------------------------------------------------------------

enum Transport {
    Http(HttpTransport),
    Stdio(StdioTransport),
}

pub struct Connection {
    transport: Transport,
    next_id: AtomicU64,
}

impl Connection {
    async fn request(&self, method: &str, params: Value, timeout: Duration) -> Result<Value, RpcError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let fut = async {
            match &self.transport {
                Transport::Http(t) => t.request(id, method, params).await,
                Transport::Stdio(t) => t.request(id, method, params).await,
            }
        };
        match tokio::time::timeout(timeout, fut).await {
            Ok(r) => r,
            Err(_) => Err(RpcError::Other(format!(
                "MCP request `{}` timed out after {}s",
                method,
                timeout.as_secs()
            ))),
        }
    }

    async fn notify(&self, method: &str, params: Value) -> Result<(), RpcError> {
        match &self.transport {
            Transport::Http(t) => t.notify(method, params).await,
            Transport::Stdio(t) => t.notify(method, params).await,
        }
    }

    /// initialize + notifications/initialized. Returns the initialize result.
    async fn initialize(&self) -> Result<Value, RpcError> {
        if let Transport::Http(t) = &self.transport {
            *t.session_id.lock().await = None;
            *t.protocol_version.lock().await = None;
        }
        let result = self
            .request(
                "initialize",
                json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "olly", "version": env!("CARGO_PKG_VERSION") }
                }),
                CONNECT_TIMEOUT,
            )
            .await?;
        if let Transport::Http(t) = &self.transport {
            if let Some(pv) = result.get("protocolVersion").and_then(|v| v.as_str()) {
                *t.protocol_version.lock().await = Some(pv.to_string());
            }
        }
        if let Err(e) = self.notify("notifications/initialized", json!({})).await {
            warn!("MCP initialized notification failed: {:?}", e);
        }
        Ok(result)
    }

    async fn list_tools(&self) -> Result<Vec<Value>, RpcError> {
        let mut tools = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..50 {
            let params = match &cursor {
                Some(c) => json!({ "cursor": c }),
                None => json!({}),
            };
            let page = self.request("tools/list", params, CONNECT_TIMEOUT).await?;
            if let Some(items) = page.get("tools").and_then(|t| t.as_array()) {
                tools.extend(items.iter().cloned());
            }
            cursor = page.get("nextCursor").and_then(|c| c.as_str()).map(String::from);
            if cursor.is_none() {
                break;
            }
        }
        Ok(tools)
    }

    async fn close(&self) {
        match &self.transport {
            Transport::Http(t) => t.close().await,
            Transport::Stdio(t) => t.close().await,
        }
    }
}

#[derive(Default)]
pub struct McpState {
    connections: Mutex<HashMap<String, Arc<Connection>>>,
}

async fn get_connection(state: &McpState, server_id: &str) -> Result<Arc<Connection>, String> {
    state
        .connections
        .lock()
        .await
        .get(server_id)
        .cloned()
        .ok_or_else(|| format!("MCP server `{}` is not connected", server_id))
}

/// Open a transport, initialize, and list tools.
/// Returns the connection and `{ serverInfo, instructions, protocolVersion, tools }`.
async fn connect(config: &ServerConfig) -> Result<(Arc<Connection>, Value), String> {
    let transport = match config {
        ServerConfig::Http { url } => Transport::Http(HttpTransport::new(url.trim().to_string())?),
        ServerConfig::Stdio { command, args, env } => {
            Transport::Stdio(StdioTransport::spawn(command.trim(), args, env)?)
        }
    };
    let conn = Arc::new(Connection { transport, next_id: AtomicU64::new(1) });

    let setup = async {
        let init = conn.initialize().await?;
        let has_tools = init.get("capabilities").and_then(|c| c.get("tools")).is_some();
        let tools = if has_tools { conn.list_tools().await? } else { Vec::new() };
        Ok::<_, RpcError>(json!({
            "serverInfo": init.get("serverInfo").cloned().unwrap_or(Value::Null),
            "instructions": init.get("instructions").cloned().unwrap_or(Value::Null),
            "protocolVersion": init.get("protocolVersion").cloned().unwrap_or(Value::Null),
            "tools": tools,
        }))
    };
    match setup.await {
        Ok(info) => Ok((conn, info)),
        Err(e) => {
            conn.close().await;
            Err(e.into_string())
        }
    }
}

async fn call_tool(conn: &Connection, name: &str, arguments: Value) -> Result<Value, String> {
    let arguments = if arguments.is_object() { arguments } else { json!({}) };
    let params = json!({ "name": name, "arguments": arguments });
    match conn.request("tools/call", params.clone(), CALL_TIMEOUT).await {
        Ok(r) => Ok(r),
        Err(RpcError::SessionExpired) => {
            // Server restarted or dropped the session: re-initialize once and retry.
            info!("MCP session expired; re-initializing");
            conn.initialize().await.map_err(RpcError::into_string)?;
            conn.request("tools/call", params, CALL_TIMEOUT)
                .await
                .map_err(RpcError::into_string)
        }
        Err(e) => Err(e.into_string()),
    }
}

/// Connect (or reconnect) a server and return its initialize info + tools:
/// `{ serverInfo, instructions, protocolVersion, tools }`.
#[tauri::command]
pub async fn mcp_connect(
    state: tauri::State<'_, McpState>,
    server_id: String,
    config: ServerConfig,
) -> Result<Value, String> {
    info!("MCP connect {} {:?}", server_id, config);
    if let Some(old) = state.connections.lock().await.remove(&server_id) {
        old.close().await;
    }
    let (conn, info) = connect(&config).await?;
    state.connections.lock().await.insert(server_id, conn);
    Ok(info)
}

#[tauri::command]
pub async fn mcp_list_tools(state: tauri::State<'_, McpState>, server_id: String) -> Result<Vec<Value>, String> {
    let conn = get_connection(&state, &server_id).await?;
    conn.list_tools().await.map_err(RpcError::into_string)
}

/// Call a tool. Returns the raw MCP `CallToolResult` (`{content, isError, structuredContent?}`).
/// Protocol/transport failures are returned as `Err(String)`.
#[tauri::command]
pub async fn mcp_call_tool(
    state: tauri::State<'_, McpState>,
    server_id: String,
    name: String,
    arguments: Value,
) -> Result<Value, String> {
    let conn = get_connection(&state, &server_id).await?;
    call_tool(&conn, &name, arguments).await
}

#[tauri::command]
pub async fn mcp_disconnect(state: tauri::State<'_, McpState>, server_id: String) -> Result<(), String> {
    if let Some(conn) = state.connections.lock().await.remove(&server_id) {
        conn.close().await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live check against a running server, e.g.
    /// `MCP_TEST_URL=http://127.0.0.1:4510/mcp cargo test mcp_live -- --ignored --nocapture`
    /// or `MCP_TEST_CMD="node server.mjs"` for a stdio server.
    #[tokio::test]
    #[ignore]
    async fn mcp_live() {
        let config = if let Ok(url) = std::env::var("MCP_TEST_URL") {
            ServerConfig::Http { url }
        } else {
            let cmd = std::env::var("MCP_TEST_CMD").expect("set MCP_TEST_URL or MCP_TEST_CMD");
            let mut parts = cmd.split_whitespace().map(String::from);
            ServerConfig::Stdio { command: parts.next().unwrap(), args: parts.collect(), env: HashMap::new() }
        };
        let (conn, info) = connect(&config).await.expect("connect");
        let tools = info["tools"].as_array().unwrap();
        println!("server: {}", info["serverInfo"]);
        println!("instructions: {} chars", info["instructions"].as_str().map(|s| s.len()).unwrap_or(0));
        println!("tools ({}): {:?}", tools.len(), tools.iter().map(|t| t["name"].as_str().unwrap_or("")).collect::<Vec<_>>());
        if let Ok(tool) = std::env::var("MCP_TEST_TOOL") {
            let args: Value = serde_json::from_str(&std::env::var("MCP_TEST_ARGS").unwrap_or("{}".into())).unwrap();
            let result = call_tool(&conn, &tool, args).await;
            let text = format!("{:?}", result);
            println!("call {} -> {}", tool, text.chars().take(600).collect::<String>());
        }
        conn.close().await;
    }

    #[tokio::test]
    async fn unreachable_server_is_an_error() {
        let config = ServerConfig::Http { url: "http://127.0.0.1:9/mcp".into() };
        let err = connect(&config).await.err().expect("should fail");
        assert!(err.contains("Cannot reach"), "{}", err);
    }
}
