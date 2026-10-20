// Interactive Web Portal HTML & Styling for Mock Dealer & Shopper Hub
export function getPortalHtml(port = 3456) {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>AutoShopper MCP & Anonymous Dealer Relay</title>
  <style>
    :root {
      --bg: #0f172a;
      --card-bg: #1e293b;
      --border: #334155;
      --text: #f8fafc;
      --text-muted: #94a3b8;
      --accent: #38bdf8;
      --accent-hover: #0284c7;
      --great-deal: #22c55e;
      --good-deal: #38bdf8;
      --fair-deal: #eab308;
      --dealer-tag: #f43f5e;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; }
    body { background: var(--bg); color: var(--text); padding: 24px; min-height: 100vh; }
    header { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: 16px; margin-bottom: 24px; }
    .logo { display: flex; align-items: center; gap: 12px; font-size: 20px; font-weight: 700; color: var(--accent); }
    .badge { padding: 4px 10px; border-radius: 9999px; font-size: 12px; font-weight: 600; text-transform: uppercase; }
    .badge-mcp { background: #6366f1; color: white; }
    .tabs { display: flex; gap: 10px; margin-bottom: 24px; border-bottom: 1px solid var(--border); }
    .tab { padding: 10px 18px; cursor: pointer; color: var(--text-muted); font-weight: 500; border-bottom: 2px solid transparent; transition: all 0.2s; }
    .tab.active { color: var(--accent); border-bottom-color: var(--accent); }
    .tab:hover { color: var(--text); }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(360px, 1fr)); gap: 20px; }
    .card { background: var(--card-bg); border: 1px solid var(--border); border-radius: 12px; padding: 20px; display: flex; flex-direction: column; gap: 12px; }
    .card-title { font-size: 18px; font-weight: 600; }
    .card-subtitle { color: var(--text-muted); font-size: 13px; }
    .price-row { display: flex; align-items: baseline; gap: 10px; }
    .price { font-size: 24px; font-weight: 700; color: #fff; }
    .deal-badge { padding: 3px 8px; border-radius: 6px; font-size: 12px; font-weight: 600; }
    .deal-great { background: rgba(34, 197, 94, 0.2); color: #4ade80; border: 1px solid #22c55e; }
    .deal-good { background: rgba(56, 189, 248, 0.2); color: #38bdf8; border: 1px solid #38bdf8; }
    .deal-fair { background: rgba(234, 179, 8, 0.2); color: #facc15; border: 1px solid #eab308; }
    .meta-list { display: flex; flex-wrap: wrap; gap: 8px; font-size: 12px; color: var(--text-muted); margin-top: 4px; }
    .meta-tag { background: #0f172a; padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border); }
    button { background: var(--accent); color: #0f172a; font-weight: 600; border: none; padding: 10px 14px; border-radius: 8px; cursor: pointer; transition: background 0.2s; }
    button:hover { background: var(--accent-hover); }
    button.secondary { background: #334155; color: white; }
    button.secondary:hover { background: #475569; }
    button.danger { background: var(--dealer-tag); color: white; }
    
    /* Dealer simulation panel */
    .dealer-banner { background: #3b0764; border: 1px solid #a855f7; border-radius: 8px; padding: 12px 16px; margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center; }
    .inbox-item { background: #0f172a; border: 1px solid var(--border); border-radius: 8px; padding: 16px; margin-bottom: 12px; }
    .inbox-header { display: flex; justify-content: space-between; margin-bottom: 8px; font-size: 13px; color: var(--text-muted); }
    .inbox-body { font-size: 14px; line-height: 1.5; color: #e2e8f0; white-space: pre-line; background: #1e293b; padding: 12px; border-radius: 6px; margin: 10px 0; }
    .reply-box { margin-top: 12px; display: flex; flex-direction: column; gap: 8px; }
    textarea { width: 100%; height: 90px; background: #0f172a; color: white; border: 1px solid var(--border); border-radius: 6px; padding: 10px; font-size: 14px; }
    .quick-replies { display: flex; gap: 8px; flex-wrap: wrap; }
    .quick-chip { background: #334155; color: var(--text); border: none; padding: 5px 10px; border-radius: 4px; font-size: 12px; cursor: pointer; }
    .quick-chip:hover { background: #475569; }

    /* Code block */
    pre { background: #020617; border: 1px solid var(--border); border-radius: 8px; padding: 16px; color: #a5f3fc; overflow-x: auto; font-size: 13px; }
  </style>
</head>
<body>

  <header>
    <div class="logo">
      <span>🚗 AutoShopper WebMCP</span>
      <span class="badge badge-mcp">MCP Active on :${port}</span>
    </div>
    <div>
      <span style="font-size: 13px; color: var(--text-muted);">Connector Endpoint: <code>http://localhost:${port}/sse</code></span>
    </div>
  </header>

  <div class="tabs">
    <div class="tab active" onclick="switchTab('inventory')">Car Listings Catalog</div>
    <div class="tab" onclick="switchTab('dealer')">🏢 Mock Dealership Inbox (<span id="inbox-count">0</span>)</div>
    <div class="tab" onclick="switchTab('aliases')">🎭 Anonymous Personas & Relay (<span id="alias-count">0</span>)</div>
    <div class="tab" onclick="switchTab('mcp')">🔌 MCP Integration Config</div>
  </div>

  <!-- TAB 1: Inventory -->
  <div id="tab-inventory" class="tab-content">
    <div style="margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center;">
      <p style="color: var(--text-muted);">Simulated live CarGurus vehicle listings accessible to your AI assistant via MCP tools.</p>
    </div>
    <div id="inventory-grid" class="grid">
      <!-- Injected via JS -->
    </div>
  </div>

  <!-- TAB 2: Dealer Portal -->
  <div id="tab-dealer" class="tab-content" style="display: none;">
    <div class="dealer-banner">
      <div>
        <strong>Dealership CRM Simulator</strong>
        <p style="font-size: 13px; color: #e9d5ff;">Inquiries sent by your AI agent appear here. Reply as the dealer to test the AI's inbox tool!</p>
      </div>
      <button class="secondary" onclick="refreshDealerInbox()">↻ Refresh Inquiries</button>
    </div>
    <div id="dealer-inquiries">
      <p style="color: var(--text-muted); text-align: center; padding: 40px;">No dealer inquiries received yet. Have your AI run <code>send_dealer_inquiry</code>!</p>
    </div>
  </div>

  <!-- TAB 3: Aliases -->
  <div id="tab-aliases" class="tab-content" style="display: none;">
    <div style="margin-bottom: 16px;">
      <p style="color: var(--text-muted);">Active masked identities generated by MCP to protect the user's real email and phone.</p>
    </div>
    <div id="aliases-list">
      <p style="color: var(--text-muted); text-align: center; padding: 40px;">No active personas created yet. Ask your AI: "Create an anonymous alias for the 2022 Honda Accord".</p>
    </div>
  </div>

  <!-- TAB 4: MCP Config -->
  <div id="tab-mcp" class="tab-content" style="display: none;">
    <h3 style="margin-bottom: 12px;">Connecting to AI Assistants (Claude Desktop, Olly, Cursor)</h3>
    <p style="color: var(--text-muted); margin-bottom: 16px;">Add this configuration to connect your assistant to this Car Shopper MCP server.</p>

    <h4 style="margin-bottom: 8px;">Option A: stdio Transport (Claude Desktop / Cursor)</h4>
    <pre><code>{
  "mcpServers": {
    "car-shopper": {
      "command": "node",
      "args": ["${process.cwd()}/car-shopper-webmcp/server.js", "--stdio"]
    }
  }
}</code></pre>

    <h4 style="margin: 20px 0 8px 0;">Option B: SSE / WebMCP Transport (Remote or Web Connector)</h4>
    <pre><code>{
  "mcpServers": {
    "car-shopper-web": {
      "url": "http://localhost:${port}/sse"
    }
  }
}</code></pre>
  </div>

  <script>
    let currentTab = 'inventory';

    function switchTab(tabId) {
      document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
      document.querySelectorAll('.tab-content').forEach(c => c.style.display = 'none');
      
      const tabElements = document.querySelectorAll('.tab');
      if (tabId === 'inventory') tabElements[0].classList.add('active');
      if (tabId === 'dealer') tabElements[1].classList.add('active');
      if (tabId === 'aliases') tabElements[2].classList.add('active');
      if (tabId === 'mcp') tabElements[3].classList.add('active');

      document.getElementById('tab-' + tabId).style.display = 'block';
      currentTab = tabId;

      if (tabId === 'dealer') refreshDealerInbox();
      if (tabId === 'aliases') refreshAliases();
    }

    async function loadInventory() {
      try {
        const res = await fetch('/api/inventory');
        const cars = await res.json();
        const container = document.getElementById('inventory-grid');
        container.innerHTML = cars.map(c => \`
          <div class="card">
            <div class="card-title">\${c.year} \${c.make} \${c.model} \${c.trim}</div>
            <div class="card-subtitle">VIN: \${c.vin} • \${c.mileage.toLocaleString()} mi</div>
            <div class="price-row">
              <span class="price">$\${c.price.toLocaleString()}</span>
              <span class="deal-badge \${c.deal_rating === 'Great Deal' ? 'deal-great' : c.deal_rating === 'Good Deal' ? 'deal-good' : 'deal-fair'}">\${c.deal_rating}</span>
            </div>
            <div class="meta-list">
              <span class="meta-tag">📍 \${c.dealer.name} (\${c.dealer.city}, \${c.dealer.state})</span>
              <span class="meta-tag">⭐ \${c.dealer.rating} (\${c.dealer.reviews_count} reviews)</span>
              <span class="meta-tag">⏱️ \${c.days_on_market} days on market</span>
            </div>
            <p style="font-size: 13px; color: #cbd5e1; margin-top: 4px;">\${c.features.slice(0, 2).join(' • ')}</p>
            <div style="margin-top: auto; padding-top: 10px; display: flex; gap: 8px;">
              <button onclick="promptCreateAlias('\${c.id}')" style="flex: 1;">Create Masked Alias</button>
            </div>
          </div>
        \`).join('');
      } catch (e) {
        console.error('Failed to load inventory', e);
      }
    }

    async function promptCreateAlias(carId) {
      const res = await fetch('/api/tools/call', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: 'create_anonymous_alias',
          arguments: { car_id: carId, buyer_display_name: 'Verified Shopper' }
        })
      });
      const data = await res.json();
      alert('Created Anonymous Alias!\\n' + JSON.stringify(data, null, 2));
      switchTab('aliases');
    }

    async function refreshDealerInbox() {
      try {
        const res = await fetch('/api/dealer/inquiries');
        const inquiries = await res.json();
        const container = document.getElementById('dealer-inquiries');
        document.getElementById('inbox-count').textContent = inquiries.length;

        if (!inquiries.length) {
          container.innerHTML = '<p style="color: var(--text-muted); text-align: center; padding: 40px;">No dealer inquiries yet. Have your AI agent call <code>send_dealer_inquiry</code>!</p>';
          return;
        }

        container.innerHTML = inquiries.map(m => \`
          <div class="inbox-item">
            <div class="inbox-header">
              <span><strong>From:</strong> \${escapeHtml(m.from)}</span>
              <span>\${new Date(m.sentAt).toLocaleTimeString()}</span>
            </div>
            <div><strong>Subject:</strong> \${escapeHtml(m.subject)}</div>
            <div class="inbox-body">\${escapeHtml(m.body)}</div>
            
            <div class="reply-box">
              <div class="quick-replies">
                <span style="font-size: 12px; color: var(--text-muted); align-self: center;">Quick Presets:</span>
                <button class="quick-chip" onclick="setPresetReply('\${m.aliasId}', 'clear_otd')">Give OTD Quote ($24,500 + $495 Fee)</button>
                <button class="quick-chip" onclick="setPresetReply('\${m.aliasId}', 'pushy')">Sales Push (Ask for Phone Number)</button>
                <button class="quick-chip" onclick="setPresetReply('\${m.aliasId}', 'test_drive')">Invite for Test Drive</button>
              </div>
              <textarea id="reply-text-\${m.aliasId}" placeholder="Type dealer response here..."></textarea>
              <div style="display: flex; justify-content: flex-end;">
                <button onclick="sendDealerReply('\${m.aliasId}')">Send Reply to Shopper</button>
              </div>
            </div>
          </div>
        \`).join('');
      } catch (e) {
        console.error('Failed to load dealer inbox', e);
      }
    }

    function setPresetReply(aliasId, type) {
      const textarea = document.getElementById('reply-text-' + aliasId);
      if (!textarea) return;

      if (type === 'clear_otd') {
        textarea.value = "Hello! Thanks for reaching out. Yes, the vehicle is available. Here is the requested Out-The-Door breakdown:\\n- Agreed Price: $24,500\\n- Documentation Fee: $495\\n- State Sales Tax (6.25%): $1,562.19\\n- Title & Reg: $135\\nTotal Out-The-Door: $26,692.19.\\nNo additional accessories or mandatory packages. Let me know if you would like to proceed!";
      } else if (type === 'pushy') {
        textarea.value = "Hi! Yes this car is getting a lot of attention today! What is the best cell phone number to reach you at? When can you come down to the lot to take a look? Prices are only valid in person.";
      } else if (type === 'test_drive') {
        textarea.value = "Good morning! The car has passed our 150-point inspection and is parked right out front. Can I hold the keys for you this afternoon at 2:00 PM for a quick test drive?";
      }
    }

    async function sendDealerReply(aliasId) {
      const textarea = document.getElementById('reply-text-' + aliasId);
      const text = textarea ? textarea.value.trim() : '';
      if (!text) {
        alert('Please enter a message before sending.');
        return;
      }

      const res = await fetch('/api/dealer/reply', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          aliasId,
          message: text
        })
      });

      if (res.ok) {
        alert('Reply sent! Your AI assistant can now run check_inbox(alias_id="' + aliasId + '") to retrieve it.');
        textarea.value = '';
        refreshDealerInbox();
        refreshAliases();
      } else {
        alert('Error sending reply');
      }
    }

    async function refreshAliases() {
      try {
        const res = await fetch('/api/aliases');
        const aliases = await res.json();
        document.getElementById('alias-count').textContent = aliases.length;
        const container = document.getElementById('aliases-list');

        if (!aliases.length) {
          container.innerHTML = '<p style="color: var(--text-muted); text-align: center; padding: 40px;">No active personas created yet.</p>';
          return;
        }

        container.innerHTML = aliases.map(a => \`
          <div class="card" style="margin-bottom: 16px;">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 700; color: var(--accent);">\${a.email}</span>
              <span class="badge" style="background: \${a.status === 'anonymous' ? '#059669' : '#d97706'}; color: white;">
                \${a.status === 'anonymous' ? '🛡️ Anonymous' : '🔓 Contact Shared'}
              </span>
            </div>
            <div style="font-size: 14px; color: #cbd5e1;">Target Vehicle: <strong>\${a.carSummary}</strong> (Dealer: \${a.dealer.name})</div>
            <div style="font-size: 13px; color: var(--text-muted);">Unread Dealer Messages: <strong>\${a.unreadCount}</strong> • Alias ID: <code>\${a.aliasId}</code></div>
          </div>
        \`).join('');
      } catch (e) {
        console.error('Failed to load aliases', e);
      }
    }

    function escapeHtml(str) {
      if (!str) return '';
      return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }

    // Initial load
    loadInventory();
    refreshDealerInbox();
    refreshAliases();
  </script>
</body>
</html>`;
}
