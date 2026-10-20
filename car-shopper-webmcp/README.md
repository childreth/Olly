# 🚗 Car Shopper WebMCP & Anonymous Dealer Outreach POC

A Proof-of-Concept demonstrating how car-shopping websites (like CarGurus) can expose **Model Context Protocol (MCP / WebMCP)** tools. This enables an AI assistant to search vehicle inventories, spin up anonymous email aliases, dispatch dealer inquiries, and analyze dealer responses—**without the buyer having to manually create disposable burner emails or expose their phone number to aggressive dealer CRM funnels.**

---

## 💡 Why This Exists (The Problem Solved)

When shopping for cars online, dealers often lock pricing transparency (window stickers, doc fees, out-the-door numbers) behind "Contact Dealer" forms. Buyers who fill out these forms get inundated with automated phone calls, texts, and relentless follow-ups.

Traditionally, buyers spend time setting up disposable email inboxes or Google Voice numbers.

With **Car Shopper MCP**:
1. **Zero-Friction Masking**: The AI assistant creates an ephemeral alias (`buyer_x9a@shopper-relay.net`) tied to that specific vehicle and dealership.
2. **Dealer Never Sees Real Identity**: Inquiries go out via the relay with the buyer's real email and phone completely hidden.
3. **Automated Price & Fee Extraction**: The assistant monitors incoming dealer replies, parses itemized prices, documentation fees, and flags aggressive sales push tactics.
4. **Controlled Reveal**: The buyer's real name and phone are only shared when the buyer explicitly instructs the assistant that they agree on price and want to book a test drive.
5. **Safe Connector Model**: Tools are connected intentionally via the standardized Model Context Protocol, respecting the security boundary separating webpage content from agent instructions.

---

## 🛠️ MCP Tools Exposed

| Tool Name | Parameters | Purpose |
|---|---|---|
| `search_cars` | `make`, `model`, `min_price`, `max_price`, `max_mileage`, `min_year`, `deal_rating`, `sort` | Search car listings like CarGurus with deal rating indicators. |
| `get_car_details` | `car_id` | Inspect complete specs, VIN, features, dealer reputation, and fee estimates. |
| `create_anonymous_alias` | `car_id`, `buyer_display_name` | Instantly provision a secure email alias linked to the car and dealer. |
| `list_active_aliases` | *none* | View all active anonymous aliases and unread message counters. |
| `send_dealer_inquiry` | `alias_id`, `subject`, `message_body`, `inquiry_type` | Send an inquiry email to the dealership through the anonymous alias. |
| `check_inbox` | `alias_id`, `unread_only` | Retrieve incoming dealer replies, auto-extracting price quotes and test drive invites. |
| `reply_to_dealer` | `alias_id`, `message_body` | Send follow-up replies back through the relay. |
| `reveal_contact_info` | `alias_id`, `full_name`, `phone_number`, `note` | Safely release real contact info to book an appointment once satisfied with price. |
| `get_conversation_thread` | `alias_id` | Review full audit trail of back-and-forth messages. |

---

## 🚀 Quick Start

### 1. Run Automated Test Suite
Verify the entire end-to-end flow in seconds:
```bash
cd car-shopper-webmcp
node test.js
```

### 2. Launch the Web Portal & Dealership Simulator
Start the interactive server (runs both the MCP connector and the web simulator):
```bash
node server.js --both --port=3456
```

Open **`http://localhost:3456/`** in your browser:
- **Car Catalog Tab**: Browse realistic vehicle listings and deal ratings.
- **Mock Dealership Inbox (`/dealer`)**: See inquiries arrive when your AI agent sends them. Test the AI by clicking quick-preset dealer replies (e.g. *"Give Out-The-Door Quote ($24,500 + $495 Fee)"* or *"Sales Push (Ask for Phone Number)"*) and watching the AI assistant pick them up via `check_inbox`!
- **Anonymous Personas Tab**: Inspect active masked aliases and thread status.

---

## 🔌 Connecting to AI Assistants

### Claude Desktop
Add this to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "car-shopper": {
      "command": "node",
      "args": [
        "/Users/chrishildreth/Documents/Github/Olly/car-shopper-webmcp/server.js",
        "--stdio"
      ]
    }
  }
}
```

### WebMCP / SSE Client (Remote Connector)
If using an assistant that connects over HTTP/SSE:
```json
{
  "mcpServers": {
    "car-shopper-web": {
      "url": "http://localhost:3456/sse"
    }
  }
}
```

---

## 💬 Example Prompt Flow to Try with Your Assistant

1. *"Search for 2022 or newer Honda Accords or Toyota RAV4s that are rated Great Deals."*
2. *"Show me the details and VIN for the 2022 Honda Accord."*
3. *"Create an anonymous alias for this car so my real email isn't shared with the dealer."*
4. *"Draft an inquiry asking the dealer for an itemized Out-The-Door price breakdown, including doc fees. Show me before sending."*
5. *"Send the inquiry."*
6. *(Open `http://localhost:3456/dealer`, click "Give OTD Quote", and send reply)*
7. *"Check my anonymous inbox to see if the dealer replied."*
8. *"The price looks fair. Share my real name (Alex) and phone ((555) 123-4567) with the dealer to book a test drive for Saturday morning."*
