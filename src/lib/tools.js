import { invoke } from "@tauri-apps/api/core";

/**
 * Tool definitions for Ollama tool calling
 * Each tool follows the OpenAI function calling schema
 */
export const tools = [
  {
    type: 'function',
    function: {
      name: 'getCalendarEvents',
      description: 'Fetch upcoming calendar events from the user\'s macOS Calendar. Use this when the user asks about their schedule, meetings, appointments, or what\'s on their calendar.',
      parameters: {
        type: 'object',
        required: ['daysAhead'],
        properties: {
          daysAhead: {
            type: 'integer',
            description: 'Number of days ahead to fetch events. Default is 14 days (2 weeks). Use 1 for today, 7 for this week, etc.',
            default: 14,
            minimum: 1,
            maximum: 90
          }
        }
      }
    }
  }
];

/**
 * Execute a tool by name with given arguments
 * @param {string} toolName - Name of the tool to execute
 * @param {object} args - Arguments to pass to the tool
 * @returns {Promise<string>} - Tool execution result as a string
 */
export async function executeTool(toolName, args) {
  console.log(`🔧 Executing tool: ${toolName}`, args);
  
  switch (toolName) {
    case 'getCalendarEvents':
      return await getCalendarEvents(args);
    
    default:
      throw new Error(`Unknown tool: ${toolName}`);
  }
}

/**
 * Fetch calendar events from the macOS Calendar plugin
 * @param {object} args - { daysAhead: number }
 * @returns {Promise<string>} - JSON string of calendar events
 */
async function getCalendarEvents(args) {
  const daysAhead = args.daysAhead || 14;
  
  try {
    // Check permission first
    const permissionStatus = await invoke("plugin:calendar|check_permission");
    console.log(`📅 Calendar permission status: ${permissionStatus}`);
    
    // If permission is not determined, try to request it
    if (permissionStatus === "prompt") {
      console.log("📅 Permission not determined, requesting...");
      const permissionResponse = await invoke("plugin:calendar|request_permission");
      console.log("📅 Permission response:", permissionResponse);
      
      if (!permissionResponse.granted) {
        return JSON.stringify({
          error: "Calendar access denied",
          message: "Calendar access was denied. Please go to System Settings > Privacy & Security > Calendars and enable access for the Olly app, then try again.",
          permissionStatus: "denied"
        });
      }
      // Permission just granted, continue to fetch
      console.log("📅 Permission granted, proceeding to fetch events");
    } else if (permissionStatus === "denied") {
      // Permission explicitly denied
      return JSON.stringify({
        error: "Calendar access denied",
        message: "Calendar access is currently denied. Please go to System Settings > Privacy & Security > Calendars and enable access for the Olly app.",
        permissionStatus: "denied"
      });
    } else if (permissionStatus !== "authorized") {
      // Unknown status
      return JSON.stringify({
        error: "Calendar access not granted",
        message: `Calendar permission status: ${permissionStatus}. Please check System Settings > Privacy & Security > Calendars.`,
        permissionStatus
      });
    }
    
    // Fetch events
    const response = await invoke("plugin:calendar|fetch_events", {
      payload: { daysAhead }
    });
    
    const events = response.events || [];
    
    if (events.length === 0) {
      return JSON.stringify({
        message: `No events found in the next ${daysAhead} days.`,
        events: []
      });
    }
    
    // Format events for better AI understanding
    const formattedEvents = events.map(event => ({
      title: event.title,
      startDate: event.startDate,
      endDate: event.endDate,
      location: event.location || null,
      notes: event.notes || null,
      isAllDay: event.isAllDay,
      isRecurring: event.isRecurring || false,
      calendar: event.calendarTitle || null
    }));
    
    // Calculate event statistics
    const recurringEvents = formattedEvents.filter(e => e.isRecurring);
    const oneTimeEvents = formattedEvents.filter(e => !e.isRecurring);
    
    const result = {
      message: `Found ${events.length} event(s) in the next ${daysAhead} days (${recurringEvents.length} recurring, ${oneTimeEvents.length} one-time). Provide a helpful summary of the most relevant events based on the user's question. Format any event data you show using code blocks for better readability.`,
      daysAhead,
      totalEvents: events.length,
      recurringCount: recurringEvents.length,
      oneTimeCount: oneTimeEvents.length,
      events: formattedEvents
    };
    
    // Log full response for debugging
    console.log('📅 Calendar tool response:', JSON.stringify(result, null, 2));
    console.log(`📊 Event breakdown: ${oneTimeEvents.length} one-time, ${recurringEvents.length} recurring`);
    
    return JSON.stringify(result);
    
  } catch (error) {
    console.error("Error fetching calendar events:", error);
    return JSON.stringify({
      error: "Failed to fetch calendar events",
      message: error.toString()
    });
  }
}

/**
 * Check if a model supports tool calling
 * @param {string} modelName - Name of the model
 * @returns {boolean} - True if model supports tools
 */
export function supportsToolCalling(modelName) {
  const lowerName = modelName.toLowerCase();
  
  // Models/families known to support tool calling
  const supportedPatterns = [
    // Llama 3.1+ family (tool calling added in 3.1)
    /llama-?3\.[1-9]/i,
    /llama3\.[1-9]/i,
    
    // Qwen 2.5+ family
    /qwen-?2\.[5-9]/i,
    /qwen2\.[5-9]/i,
    /qwen-?3/i,
    /qwen3/i,
    
    // Mistral family
    /mistral/i,
    /mixtral/i,
    /mistral-nemo/i,
    
    // Command R family
    /command-?r/i,
    
    // Specialized tool-calling models
    /firefunction/i,
    /functionary/i,
    /hermes.*tool/i,
    /nous.*hermes/i,
    /functiongemma/i,
    
    // Granite family (all versions)
    /granite/i,
    
    // Gemma 2 (9B+ models support tools)
    /gemma-?2.*9b/i,
    /gemma-?2.*27b/i,
    
    // DeepSeek V2+
    /deepseek.*v2/i,
    /deepseek.*v3/i,
    
    // Phi-3+ medium/large
    /phi-?3.*medium/i,
    /phi-?3.*large/i,
    
    // Aya family
    /aya-?23/i,
    /aya.*expanse/i,
    
    // SmolLM family (all versions)
    /smol/i
  ];
  
  // Check if model matches any pattern
  const matchesPattern = supportedPatterns.some(pattern => pattern.test(lowerName));
  
  if (matchesPattern) {
    console.log(`✅ Model "${modelName}" detected as tool-capable`);
    return true;
  }
  
  console.log(`⚠️ Model "${modelName}" not detected as tool-capable. Tools disabled.`);
  console.log(`   To enable tools for this model, add it to the supportedPatterns in tools.js`);
  return false;
}
