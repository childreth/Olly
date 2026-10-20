// Standard MCP JSON-RPC 2.0 Protocol Handler
import { TOOLS, executeTool } from './tools.js';

export class McpProtocolServer {
  constructor() {
    this.serverInfo = {
      name: 'car-shopper-mcp',
      version: '1.0.0'
    };
  }

  async handleMessage(message) {
    if (!message || typeof message !== 'object') {
      return this.makeError(null, -32700, 'Parse error');
    }

    const { id, method, params } = message;

    // Handle JSON-RPC notifications (no id)
    if (id === undefined || id === null) {
      if (method === 'notifications/initialized') {
        // Notification ack, no response
        return null;
      }
      return null;
    }

    try {
      switch (method) {
        case 'initialize': {
          return {
            jsonrpc: '2.0',
            id,
            result: {
              protocolVersion: '2024-11-05',
              capabilities: {
                tools: {}
              },
              serverInfo: this.serverInfo,
              instructions: 'Car Shopper MCP server. Allows searching car inventory, creating anonymous buyer aliases, sending inquiries without revealing personal contact details, and reading dealer email responses.'
            }
          };
        }

        case 'ping': {
          return {
            jsonrpc: '2.0',
            id,
            result: {}
          };
        }

        case 'tools/list': {
          return {
            jsonrpc: '2.0',
            id,
            result: {
              tools: TOOLS
            }
          };
        }

        case 'tools/call': {
          if (!params || !params.name) {
            return this.makeError(id, -32602, 'Missing tool name in params');
          }
          const toolResult = await executeTool(params.name, params.arguments || {});
          return {
            jsonrpc: '2.0',
            id,
            result: {
              content: [
                {
                  type: 'text',
                  text: typeof toolResult === 'string' ? toolResult : JSON.stringify(toolResult, null, 2)
                }
              ],
              isError: false
            }
          };
        }

        default: {
          return this.makeError(id, -32601, `Method "${method}" not found`);
        }
      }
    } catch (err) {
      return {
        jsonrpc: '2.0',
        id,
        result: {
          content: [
            {
              type: 'text',
              text: `Tool execution error: ${err.message}`
            }
          ],
          isError: true
        }
      };
    }
  }

  makeError(id, code, message) {
    return {
      jsonrpc: '2.0',
      id,
      error: {
        code,
        message
      }
    };
  }
}
