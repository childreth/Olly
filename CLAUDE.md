# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Olly is a local LLM chat application built with **Tauri**, **SvelteKit**, and **Rust**. It integrates with multiple AI services including local Ollama models, Claude API, Perplexity API, and image generation via Fal.ai. Key features include multi-modal chat (text + images), model management, secure API key storage, theme customization, and weather-based UI.

## Development Commands

**Start development server:**
```bash
npm run tauri dev
```

**Build application:**
```bash
npm run tauri build
```

**Type checking:**
```bash
npm run check
```

**Watch mode type checking:**
```bash
npm run check:watch
```

**Standard web development:**
```bash
npm run dev      # Vite dev server
npm run build    # Build web assets
npm run preview  # Preview built assets
```

## Architecture

### Frontend (SvelteKit)
- **Language**: JavaScript (NOT TypeScript) - Use plain JS syntax without type annotations
- **Type Checking**: Uses JSDoc comments + `jsconfig.json` with `svelte-check` for validation
- **Main app**: [src/routes/+page.svelte](src/routes/+page.svelte) - Primary chat interface
- **Components**: [src/lib/components/](src/lib/components/) - Reusable UI components (buttons, selectors, toggles, modals)
- **Utilities**: [src/lib/utils.js](src/lib/utils.js) - Weather API, theme management, AI-powered icon selection
- **Tauri Integration**: [src/lib/tauri.js](src/lib/tauri.js) - Tauri API wrappers
- **Static Assets**: [static/](static/) - Images, weather icons, etc.

### Backend (Rust/Tauri)
- **Main**: [src-tauri/src/main.rs](src-tauri/src/main.rs) - Tauri commands and API integrations (1765 lines)
- **Config**: [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json) - Tauri configuration
- **Cargo**: [src-tauri/Cargo.toml](src-tauri/Cargo.toml) - Rust dependencies

### Tauri Commands (Rust → Frontend)
All Tauri commands are invoked from the frontend via `invoke()` and defined in [src-tauri/src/main.rs](src-tauri/src/main.rs):

**Claude API:**
- `ask_claude()` - Non-streaming Claude API
- `stream_claude()` - Streaming Claude API (emits `claude-stream` and `claude-stream-done` events)

**Perplexity API:**
- `ask_perplexity()` - Non-streaming Perplexity API
- `stream_perplexity()` - Streaming Perplexity API (emits `perplexity-stream` and `perplexity-stream-done` events)
- `get_perplexity_models()` - List available Perplexity models

**Model Management:**
- `get_all_models()` - Get all available models from all providers

**API Key Management (Secure File Storage):**
- `store_api_key(provider, display_name, api_key)` - Store API key securely
- `get_api_key(provider)` - Retrieve API key
- `list_api_key_providers()` - List all providers with stored keys
- `delete_api_key(provider)` - Delete API key
- `get_provider_info(provider)` - Get provider metadata
- `validate_api_key(provider, api_key)` - Validate API key by testing the API
- `migrate_api_keys()` - Migrate keys from old storage (no-op in current version)
- `migrate_claude_key()` - Manual Claude key migration from env/config file

**Debug/Test Commands:**
- `debug_api_keys()` - Debug keyring and file storage
- `test_keyring()` - Test keyring functionality
- `test_store_load()` - Test hybrid storage
- `test_exact_keyring()` - Test exact keyring service/key

**Utility:**
- `greet(name)` - Basic greeting (example command)
- `get_env(name)` - Get environment variable

### API Key Storage Architecture
API keys are stored using **file-based storage** at `~/.olly/keys/` with simple XOR obfuscation:
- Storage location: `~/.olly/keys/{provider}.key`
- Keys are XOR-encoded (not real encryption, just basic obfuscation)
- Providers: `claude`, `perplexity`, `openai`
- Auto-migration: Keys in `~/.olly/config.env` or environment variables are migrated on startup
- Emits `api-keys-migrated` event to frontend on successful migration

**Note**: macOS keyring was attempted but had reliability issues, so file storage is used directly.

### AI Model Integration
- **Ollama**: Local LLM hosting on `http://localhost:11434` (accessed via `ollama/browser` package)
- **Claude API**: External API via Anthropic (requires API key, supports web search tool)
- **Perplexity API**: External API for search-powered responses
- **Image Generation**: Fal.ai integration for Flux models
- **Weather Icons**: AI-powered icon selection using local Gemma model (gemma3:1b)

### Key Features
- **Multi-modal chat**: Text and image inputs supported
- **Model switching**: Dynamic model selection (resets conversation on change)
- **Streaming responses**: Real-time token streaming for Claude and Perplexity
- **Theme system**: Dark/light mode with customizable accent colors (HSL-based CSS variables)
- **Weather integration**: Location-based weather with AI-selected icons
- **Model management**: List, select, and delete local Ollama models
- **Toast notifications**: User feedback for migrations and errors
- **Secure API keys**: File-based storage with migration from environment variables

## Configuration

### Environment Variables (Legacy)
- `CLAUDE_API_KEY` - Migrated to secure file storage on first run
- `PERPLEXITY_API_KEY` - Migrated to secure file storage on first run
- Config file: `~/.olly/config.env` (alternative to env vars, also migrated)

### API Endpoints
- **Ollama**: `http://localhost:11434`
- **Claude**: `https://api.anthropic.com/v1/messages`
- **Perplexity**: `https://api.perplexity.ai/chat/completions`
- **Weather**: OpenStreetMap Nominatim + weather.gov

### Secure Storage Locations
- **API Keys**: `~/.olly/keys/{provider}.key` (XOR-encoded)
- **Config**: `~/.olly/config.env` (legacy, migrated on startup)

## Dependencies

**Frontend (package.json):**
- `@tauri-apps/api` - Tauri frontend integration
- `@tauri-apps/plugin-dialog` - File/dialog APIs
- `ollama/browser` - Ollama browser client
- `marked` - Markdown parsing
- `@fal-ai/serverless-client` - Image generation
- `svelte` - UI framework
- `@sveltejs/kit` - SvelteKit framework
- `@sveltejs/adapter-static` - Static site generation for Tauri

**Backend (Cargo.toml):**
- `tauri` - Desktop app framework
- `reqwest` - HTTP client (with `json` feature)
- `serde_json` - JSON handling
- `tokio` - Async runtime (with `full` features)
- `chrono` - Date/time handling
- `keyring` - Keyring access (not actively used due to macOS issues)
- `dotenvy` - Environment variable loading
- `log`, `env_logger` - Logging
- `futures-util` - Stream utilities
- `objc`, `cocoa` - macOS native API access

## File Structure Notes
- Static assets in `static/` (includes weather icons in `static/weather-icons/`)
- Component library in `src/lib/components/`
- Tauri config: `src-tauri/tauri.conf.json`
- Rust source: `src-tauri/src/main.rs` (single file architecture)
- Build artifacts: `src-tauri/target/`

## Coding Standards

### JavaScript/Svelte Components
- **NO TypeScript**: This project uses JavaScript, not TypeScript
- **NO Type Annotations**: Never use `param: type` syntax in function parameters or variables
- **NO `lang="ts"`**: Always use `<script>` not `<script lang="ts">`
- **Component Props**: Use `export let propName = defaultValue;` format
- **Type Checking**: Relies on JSDoc comments and `svelte-check` for validation
- **Function Signatures**: Use plain JavaScript function syntax without type annotations

### Button Component Requirements
All Button components MUST include an `icon` prop (can be empty string `""`):
```javascript
<Button label="Save" icon="" />  // Valid
<Button label="Delete" icon="trash" />  // Valid
```
Required props: `label`, `icon` (others like `type`, `disabled` are optional)

### Rust Backend
- Single-file architecture in `main.rs` (1765 lines)
- Uses Tauri's `#[tauri::command]` macro for frontend-callable functions
- Async/await with Tokio runtime
- Structured logging with `log` and `env_logger`
- API module structure: `api_keys` module for key management (lines 13-189)

## Event System (Tauri)

Frontend listens to backend events using `listen()`:
- `claude-stream` - Streaming text chunks from Claude
- `claude-stream-done` - Final complete response from Claude
- `perplexity-stream` - Streaming text chunks from Perplexity
- `perplexity-stream-done` - Final complete response from Perplexity
- `api-keys-migrated` - API keys migrated to secure storage (shows toast)

Frontend emits these events from backend using `window.emit()`.

## AI Model Details

### Claude
- Default model: `claude-3-7-sonnet-20250219`
- Supports web search tool (`web_search_20250305`)
- Max tokens: 1024, Temperature: 0.0

### Perplexity Models
- `sonar-deep-research` - Deep research with comprehensive analysis
- `sonar-reasoning-pro` - Advanced reasoning capabilities
- `sonar-reasoning` - Core reasoning model
- `sonar-pro` - Professional grade search and chat
- `sonar` - Standard search and chat model

### Weather Icon Selection
Uses local `gemma3:1b` model to map weather conditions to icon names:
- Model runs via Ollama locally
- Temperature: 0 (deterministic)
- Returns JSON: `{ "iconName": "icon_name" }`
- Fallback: `sad_face.svg`

## Common Patterns

### Invoking Tauri Commands
```javascript
import { invoke } from "@tauri-apps/api/tauri";

// Non-streaming
const response = await invoke('ask_claude', {
  model: 'claude-3-7-sonnet-20250219',
  prompt: 'Hello',
  messages: []
});

// Streaming (requires event listener)
await invoke('stream_claude', {
  model: 'claude-3-7-sonnet-20250219',
  prompt: 'Hello',
  messages: []
});
```

### Listening to Events
```javascript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("claude-stream", (event) => {
  console.log("Received chunk:", event.payload);
});
```

### Theme Management
Theme uses CSS custom properties with HSL color system:
- `--hue` - Main color hue (0-360)
- Saved in localStorage as hex color
- Theme mode (`light`/`dark`) saved in localStorage
- Functions: `toggleTheme()`, `initTheme()` in `utils.js`

### API Key Management Flow
1. User adds key in Settings modal
2. Frontend calls `store_api_key(provider, display_name, api_key)`
3. Backend validates with `validate_api_key(provider, api_key)`
4. If valid, stores in `~/.olly/keys/{provider}.key`
5. Frontend retrieves with `get_api_key(provider)` when needed
6. Backend auto-migrates legacy keys from `~/.olly/config.env` on startup