# Olly - Local LLM Chat

Personal project to continue self-development and exploration with AI. I'm building out a local AI assistant app using Ollama, SvelteKit, and Tauri. Ollama is a sweet open-source project for running LLMs privately on a local machines, while Tauri, similar to Electron, is a lightweight framework for building fast, cross-platform desktop apps using web technologies and Rust.

So far, have the basics of the app set up: Ollama serving the LLMs via the API, rendering markdown, added conversational and vision capabilities, implementing basic model and theme management, and even having AI determine the appropriate UI iconography based on the weather.

npm run tauri dev
npm run tauri build

## MCP tools (local)

Olly can connect to [MCP](https://modelcontextprotocol.io) servers and let local Ollama models use their tools. Everything stays on your machine: Olly talks to the server directly, and the model runs in Ollama.

1. Start an MCP server, e.g. [Weft](https://github.com/childreth/Weft) (`pnpm dev` serves MCP at `http://127.0.0.1:4510/mcp`).
2. In Olly open **Settings → MCP servers**, click **Add Weft** (or enter any name + Streamable HTTP URL, or a stdio command such as `npx -y @modelcontextprotocol/server-everything`).
3. Pick a tool-capable model (Ollama lists `tools` among its capabilities, e.g. `qwen3.5:4b-mlx`, `gemma4:12b-nvfp4`) and ask something like "What artboards are on the canvas?".

Tool calls show inline in the chat (arguments, collapsible result, errors); **Stop** ends a running tool loop. Models without tool support chat as before. Enabled servers are retried automatically if they were down when Olly started.

Areas to explore next:

- Using Tauri to access native system APIs to integrate with other desktop apps.
- Tool calling to interact with external functions and APIs, e.g., adding search. (MCP servers: done, see above.)
- RAG and Conversation history
- Canvas interactions to create more multimodal experiences, like drawing to generate or edit.

Doesn’t play games as well as WOPR...yet 🙂. Planning to eventually distribute it on the App Store. You can setup locally with Ollama and get some Dad jokes.

### [YouTube Demo](https://youtu.be/j8HyOpyCduw)

![image](/static/Olly.gif)


