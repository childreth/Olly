#!/usr/bin/env node
// Car Shopper MCP Server & Web Portal
// Supports stdio transport and HTTP / SSE transport (WebMCP)
import http from 'node:http';
import readline from 'node:readline';
import crypto from 'node:crypto';
import { McpProtocolServer } from './mcp_server.js';
import { VEHICLES } from './inventory.js';
import { relayManager } from './relay.js';
import { executeTool } from './tools.js';
import { getPortalHtml } from './portal_html.js';

const mcpServer = new McpProtocolServer();

// Active SSE client sessions
const sseSessions = new Map(); // sessionId -> http response

// Parse CLI flags
const args = process.argv.slice(2);
let runStdio = args.includes('--stdio');
let runHttp = args.includes('--http') || args.includes('--both');
let port = 3456;

for (const arg of args) {
  if (arg.startsWith('--port=')) {
    port = parseInt(arg.split('=')[1], 10);
  }
}

// Default mode: if not connected to a TTY on stdin, assume stdio MCP client
if (!runStdio && !runHttp) {
  if (!process.stdin.isTTY) {
    runStdio = true;
  } else {
    // When run directly from terminal, run both so user can view web UI and test stdio
    runHttp = true;
    runStdio = true;
  }
}

// --------------------------------------------------------------------------
// 1. stdio Transport (For Claude Desktop, Cursor, CLI agents)
// --------------------------------------------------------------------------
if (runStdio) {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
  });

  rl.on('line', async (line) => {
    const trimmed = line.trim();
    if (!trimmed) return;

    try {
      const request = JSON.parse(trimmed);
      const response = await mcpServer.handleMessage(request);
      if (response !== null) {
        process.stdout.write(JSON.stringify(response) + '\n');
      }
    } catch (err) {
      process.stdout.write(JSON.stringify({
        jsonrpc: '2.0',
        id: null,
        error: { code: -32700, message: 'Invalid JSON: ' + err.message }
      }) + '\n');
    }
  });

  process.stderr.write(`[car-shopper-mcp] stdio transport initialized.\n`);
}

// --------------------------------------------------------------------------
// 2. HTTP / WebMCP / SSE Transport & Interactive Portal
// --------------------------------------------------------------------------
if (runHttp) {
  const server = http.createServer(async (req, res) => {
    const url = new URL(req.url, `http://${req.headers.host || 'localhost'}`);

    // CORS headers for WebMCP connections
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

    if (req.method === 'OPTIONS') {
      res.writeHead(204);
      res.end();
      return;
    }

    // Static HTML Portal
    if (req.method === 'GET' && (url.pathname === '/' || url.pathname === '/dealer')) {
      res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
      res.end(getPortalHtml(port));
      return;
    }

    // SSE Endpoint for MCP clients
    if (req.method === 'GET' && url.pathname === '/sse') {
      const sessionId = crypto.randomUUID();
      res.writeHead(200, {
        'Content-Type': 'text/event-stream',
        'Cache-Control': 'no-cache',
        'Connection': 'keep-alive'
      });

      // Send endpoint notification
      res.write(`event: endpoint\ndata: /message?sessionId=${sessionId}\n\n`);
      sseSessions.set(sessionId, res);

      req.on('close', () => {
        sseSessions.delete(sessionId);
      });
      return;
    }

    // Message handler for SSE session
    if (req.method === 'POST' && url.pathname === '/message') {
      const sessionId = url.searchParams.get('sessionId');
      const body = await parseJsonBody(req);
      const mcpResponse = await mcpServer.handleMessage(body);

      // If SSE connection is open for this session, push to SSE stream
      const sseRes = sseSessions.get(sessionId);
      if (sseRes && mcpResponse) {
        sseRes.write(`event: message\ndata: ${JSON.stringify(mcpResponse)}\n\n`);
      }

      // Also return in HTTP response
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(mcpResponse || { status: 'ok' }));
      return;
    }

    // REST APIs for web dashboard
    if (req.method === 'GET' && url.pathname === '/api/inventory') {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(VEHICLES));
      return;
    }

    if (req.method === 'GET' && url.pathname === '/api/aliases') {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(relayManager.getAllAliases()));
      return;
    }

    if (req.method === 'GET' && url.pathname === '/api/dealer/inquiries') {
      // Return all inquiries received across dealers
      const allInquiries = [];
      for (const [dealerId, msgs] of relayManager.dealerInboxes.entries()) {
        allInquiries.push(...msgs);
      }
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(allInquiries));
      return;
    }

    if (req.method === 'POST' && url.pathname === '/api/dealer/reply') {
      try {
        const { aliasId, senderName, subject, message } = await parseJsonBody(req);
        const replyMsg = relayManager.receiveDealerReply(aliasId, senderName, subject, message);
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ success: true, message: replyMsg }));
      } catch (err) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: err.message }));
      }
      return;
    }

    if (req.method === 'POST' && url.pathname === '/api/tools/call') {
      try {
        const { name, arguments: toolArgs } = await parseJsonBody(req);
        const result = await executeTool(name, toolArgs);
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(result));
      } catch (err) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: err.message }));
      }
      return;
    }

    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Not Found' }));
  });

  server.listen(port, () => {
    process.stderr.write(`[car-shopper-mcp] HTTP & WebMCP server listening on http://localhost:${port}\n`);
    process.stderr.write(`[car-shopper-mcp] Open http://localhost:${port}/ in your browser to view the Car Catalog & Mock Dealership CRM\n`);
  });
}

function parseJsonBody(req) {
  return new Promise((resolve, reject) => {
    let raw = '';
    req.on('data', chunk => { raw += chunk; });
    req.on('end', () => {
      try {
        resolve(raw ? JSON.parse(raw) : {});
      } catch (e) {
        reject(e);
      }
    });
    req.on('error', reject);
  });
}
