# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Olly is a local LLM chat application built with **Tauri**, **SvelteKit**, and **Rust**. It integrates with multiple AI services including local Ollama models, Claude API, and image generation via Fal.ai. Key features include multi-modal chat (text + images), model management, theme customization, and weather-based UI.

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
- **Type Checking**: Uses JSDoc comments and svelte-check for type validation
- **Main app**: `src/routes/+page.svelte` - Primary chat interface
- **Components**: `src/lib/components/` - Reusable UI components (buttons, selectors, toggles)
- **Utilities**: `src/lib/utils.js` - Weather API, theme management, icon selection via AI
- **Tauri Integration**: `src/lib/tauri.js` - Tauri API wrappers (currently empty)

### Backend (Rust/Tauri)
- **Main**: `src-tauri/src/main.rs` - Tauri commands and API integrations
- **Commands**: 
  - `greet()` - Basic greeting
  - `get_env()` - Environment variable access
  - `ask_claude()` - Non-streaming Claude API
  - `stream_claude()` - Streaming Claude API with frontend events
  - Perplexity API integration (ask_perplexity, stream_perplexity)

### AI Model Integration
- **Ollama**: Local LLM hosting on `localhost:11434`
- **Claude API**: External API via Anthropic (requires `CLAUDE_API_KEY`)
- **Image Generation**: Fal.ai integration for Flux models
- **Weather Icons**: AI-powered icon selection using local Gemma model

### Key Features
- **Multi-modal chat**: Text and image inputs supported
- **Model switching**: Dynamic model selection with conversation reset
- **Streaming responses**: Real-time token streaming for better UX
- **Theme system**: Dark/light mode with customizable accent colors
- **Weather integration**: Location-based weather with AI-selected icons
- **Model management**: List, select, and delete local Ollama models

## Configuration

### Environment Variables
- `CLAUDE_API_KEY` - Required for Claude API access
- Config file: `~/.olly/config.env` (alternative to env vars)

### API Endpoints
- Ollama: `http://localhost:11434`
- Weather: OpenStreetMap Nominatim + weather.gov
- Rick & Morty API: Test endpoint for HTTP functionality

## Dependencies

**Frontend:**
- `@tauri-apps/api` - Tauri frontend integration
- `ollama/browser` - Ollama browser client
- `marked` - Markdown parsing
- `@fal-ai/serverless-client` - Image generation

**Backend:**
- `tauri` - Desktop app framework
- `reqwest` - HTTP client
- `serde_json` - JSON handling
- `tokio` - Async runtime
- `chrono` - Date/time handling

## File Structure Notes
- Static assets duplicated in `src/lib/images/` and `static/images/`
- Weather icons specifically in `static/weather-icons/`
- Tauri config: `src-tauri/tauri.conf.json`
- Build artifacts: `build/` directory

## Coding Standards

### JavaScript/Svelte Components
- **NO TypeScript**: This project uses JavaScript, not TypeScript
- **NO Type Annotations**: Never use `param: type` syntax in function parameters or variables
- **NO `lang="ts"`**: Always use `<script>` not `<script lang="ts">`
- **Component Props**: Use `export let propName = defaultValue;` format
- **Type Checking**: Relies on JSDoc comments and svelte-check for validation
- **Function Signatures**: Use plain JavaScript function syntax without type annotations

### Button Component Requirements
- All Button components MUST include an `icon` prop (can be empty string `""`)
- Required props: `label`, `icon` (others are optional)