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
      description: 'Fetch upcoming calendar events from the user\'s macOS Calendar. Use this when the user asks about their schedule, meetings, appointments, or what\'s on their calendar. If the user ask involves traveling addtional can use getWeather tool to determine good or bad travel conditions.',
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
  },
  {
    type: 'function',
    function: {
      name: 'createCalendarEvent',
      description: 'Create a new calendar event in the user\'s macOS Calendar. Use this when the user asks to schedule, create, or add a meeting, appointment, or event. Requires title and date/time information.',
      parameters: {
        type: 'object',
        required: ['title', 'startDate', 'endDate'],
        properties: {
          title: {
            type: 'string',
            description: 'Title/name of the event (e.g., "Meeting with Bob", "Dentist Appointment")'
          },
          startDate: {
            type: 'string',
            description: 'Event start date and time in ISO8601 format (e.g., "2026-01-15T14:00:00Z" or "2026-01-15T14:00:00-05:00"). Must include timezone.'
          },
          endDate: {
            type: 'string',
            description: 'Event end date and time in ISO8601 format (e.g., "2026-01-15T15:00:00Z"). Must be after startDate. Must include timezone.'
          },
          location: {
            type: 'string',
            description: 'Optional location of the event (e.g., "Conference Room A", "123 Main St, Boston, MA")'
          },
          notes: {
            type: 'string',
            description: 'Optional notes or description for the event'
          },
          isAllDay: {
            type: 'boolean',
            description: 'Whether this is an all-day event. Default is false.',
            default: false
          }
        }
      }
    }
  },
  {
    type: 'function',
    function: {
      name: 'checkCalendarStatus',
      description: 'Check the status of calendar permissions and available calendars. Use this when the user reports issues with calendar access or creating events, or asks to "check calendar status" or "debug calendar".',
      parameters: {
        type: 'object',
        properties: {},
        required: []
      }
    }
  },
  {
    type: 'function',
    function: {
      name: 'getWeather',
      description: 'Get weather forecast for a specific location. Returns current weather or multi-day forecast based on user intent. Use this when the user asks about weather, temperature, forecast, or conditions for any location.',
      parameters: {
        type: 'object',
        required: ['location'],
        properties: {
          location: {
            type: 'string',
            description: 'Location in "City, State" format (e.g., "Boston, MA" or "New York, NY"). Can also accept full city names like "San Francisco, California".'
          },
          days: {
            type: 'integer',
            description: 'Number of forecast days to return. Use 1 for current/today only, 2-7 for multi-day forecast. Default is 1 (current weather only).',
            default: 1,
            minimum: 1,
            maximum: 7
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
      return await getCalendarEvents(/** @type {{ daysAhead?: number }} */ (args));

    case 'checkCalendarStatus':
      return await checkCalendarStatus();
    
    case 'createCalendarEvent':
      return await createCalendarEvent(/** @type {{ title: string, startDate: string, endDate: string, location?: string, notes?: string, isAllDay?: boolean }} */ (args));
    
    case 'getWeather':
      return await getWeather(/** @type {{ location: string, days?: number }} */ (args));
    
    default:
      throw new Error(`Unknown tool: ${toolName}`);
  }
}

/**
 * Fetch calendar events from the macOS Calendar plugin
 * @param {object} args - { daysAhead: number }
 * @returns {Promise<string>} - JSON string of calendar events
 */

/**
 * Check calendar permission status and available calendars
 * @returns {Promise<string>} - JSON string of diagnostic info
 */
async function checkCalendarStatus() {
  try {
    console.log("🔍 Checking calendar diagnostics...");
    const diags = await invoke("plugin:calendar|get_diagnostics");
    console.log("📅 Calendar Diagnostics:", diags);
    
    // Format for AI readability
    return JSON.stringify({
      status: "success",
      message: "Here is the current calendar system status. Analyze this to help the user debug permission issues.",
      diagnostics: diags
    });
  } catch (error) {
    console.error("Error checking calendar status:", error);
    const errorMessage = error instanceof Error ? error.message : String(error);
    return JSON.stringify({
      status: "error", 
      message: "Failed to run diagnostics",
      error: errorMessage
    });
  }
}

/**
 * Create a calendar event in macOS Calendar
 * @param {{ title: string, startDate: string, endDate: string, location?: string, notes?: string, isAllDay?: boolean }} args
 * @returns {Promise<string>} - JSON string of creation result
 */
async function createCalendarEvent(args) {
  const { title, startDate, endDate, location, notes, isAllDay } = args;
  
  try {
    console.log(`📅 Creating calendar event: ${title}`);
    
    // Validate required fields
    if (!title || !startDate || !endDate) {
      return JSON.stringify({
        success: false,
        error: "Missing required fields: title, startDate, and endDate are required"
      });
    }
    
    // Check permission first
    const permissionStatus = await invoke("plugin:calendar|check_permission");
    console.log(`📅 Calendar permission status: ${permissionStatus}`);
    
    if (permissionStatus !== "authorized") {
      return JSON.stringify({
        success: false,
        error: "Calendar access not granted. Please grant calendar permission first.",
        permissionStatus
      });
    }
    
    // Create the event
    const response = await invoke("plugin:calendar|create_event", {
      payload: {
        title,
        startDate,
        endDate,
        location: location || null,
        notes: notes || null,
        isAllDay: isAllDay || false
      }
    });
    
    if (response.success) {
      console.log(`✅ Event created successfully: ${response.eventId}`);
      return JSON.stringify({
        success: true,
        message: `Event "${title}" created successfully in ${response.calendarTitle || 'your calendar'}`,
        eventId: response.eventId,
        calendarTitle: response.calendarTitle
      });
    } else {
      console.error(`❌ Failed to create event: ${response.error}`);
      
      // Attempt to gather diagnostics on failure
      try {
        const diags = await invoke("plugin:calendar|get_diagnostics");
        console.log("📅 Calendar Diagnostics:", diags);
        return JSON.stringify({
          success: false,
          error: response.error || "Failed to create event",
          diagnostics: diags,
          suggestion: "Please ensure you have at least one writable calendar and Olly has Full Access in System Settings."
        });
      } catch (diagError) {
        return JSON.stringify({
          success: false,
          error: response.error || "Failed to create event"
        });
      }
    }
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("Error creating calendar event:", error);
    return JSON.stringify({
      success: false,
      error: errorMessage,
      message: "Failed to create calendar event. Please ensure calendar access is granted and dates are in valid ISO8601 format."
    });
  }
}

/**
 * Fetch calendar events from the macOS Calendar plugin
 * @param {{ daysAhead?: number }} args
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
    const formattedEvents = events.map((/** @type {{ title: string, startDate: string, endDate: string, location?: string, notes?: string, isAllDay: boolean, isRecurring?: boolean, calendarTitle?: string }} */ event) => ({
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
    const recurringEvents = formattedEvents.filter((/** @type {{ isRecurring: boolean }} */ e) => e.isRecurring);
    const oneTimeEvents = formattedEvents.filter((/** @type {{ isRecurring: boolean }} */ e) => !e.isRecurring);
    
    const result = {
      message: `Found ${events.length} event(s) in the next ${daysAhead} days: ${recurringEvents.length} recurring and ${oneTimeEvents.length} one-time events. When summarizing, make sure to include BOTH recurring and one-time events. Format any event data you show using code blocks for better readability.`,
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
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("Error fetching calendar events:", error);
    return JSON.stringify({
      error: "Failed to fetch calendar events",
      message: errorMessage
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
    
    // Mistral family (all variants)
    /mistral/i,
    /mixtral/i,
    /mistral-nemo/i,
    /ministral/i,
    
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
    
    // DeepSeek (all versions including R1, V2, V3)
    /deepseek/i,
    
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

/**
 * Get weather forecast for a location
 * @param {{ location: string, days?: number }} args
 * @returns {Promise<string>} - JSON string of weather forecast
 */
async function getWeather(args) {
  const { location, days = 1 } = args;
  
  try {
    console.log(`🌤️ Fetching weather for: ${location}, days: ${days}`);
    
    // Step 1: Get coordinates from location using OpenStreetMap Nominatim API
    const { fetch: tauriFetch } = await import("@tauri-apps/plugin-http");
    
    const coordinatesResponse = await tauriFetch(
      `https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(location)}&format=json`,
      {
        method: 'GET',
        headers: {
          'User-Agent': 'Olly Weather App/1.0'
        }
      }
    );
    
    if (!coordinatesResponse.ok) {
      throw new Error(`Failed to fetch coordinates: ${coordinatesResponse.status}`);
    }
    
    const locationData = await coordinatesResponse.json();
    
    if (!locationData || locationData.length === 0) {
      return JSON.stringify({
        error: "Location not found",
        message: `Could not find coordinates for "${location}". Please try a different location format like "City, State" (e.g., "Boston, MA").`
      });
    }
    
    const lat = locationData[0].lat;
    const lon = locationData[0].lon;
    const displayName = locationData[0].display_name;
    
    console.log(`📍 Found coordinates: ${lat}, ${lon} for ${displayName}`);
    
    // Step 2: Get weather.gov grid point data
    const pointsResponse = await tauriFetch(
      `https://api.weather.gov/points/${lat},${lon}`,
      {
        method: 'GET',
        headers: {
          'User-Agent': 'Olly Weather App/1.0'
        }
      }
    );
    
    if (!pointsResponse.ok) {
      // Check if location is outside US (weather.gov only covers US)
      if (pointsResponse.status === 404) {
        return JSON.stringify({
          error: "Location not supported",
          message: `Weather.gov only provides forecasts for US locations. "${location}" appears to be outside the US coverage area.`
        });
      }
      throw new Error(`Weather.gov points API error: ${pointsResponse.status}`);
    }
    
    const pointsData = await pointsResponse.json();
    
    if (!pointsData?.properties?.forecast) {
      throw new Error('Invalid weather.gov points response structure');
    }
    
    const forecastUrl = pointsData.properties.forecast;
    console.log(`🔗 Forecast URL: ${forecastUrl}`);
    
    // Step 3: Get the full 7-day forecast
    const forecastResponse = await tauriFetch(forecastUrl, {
      method: 'GET',
      headers: {
        'User-Agent': 'Olly Weather App/1.0'
      }
    });
    
    if (!forecastResponse.ok) {
      throw new Error(`Weather.gov forecast API error: ${forecastResponse.status}`);
    }
    
    const forecastData = await forecastResponse.json();
    
    if (!forecastData?.properties?.periods) {
      throw new Error('Invalid weather.gov forecast response structure');
    }
    
    const allPeriods = forecastData.properties.periods;
    
    // Step 4: Filter periods based on requested days
    // Each day has 2 periods (day and night), so we need days * 2 periods
    const requestedPeriods = allPeriods.slice(0, days * 2);
    
    // Format the weather data
    const formattedPeriods = requestedPeriods.map((/** @type {{ name: string, temperature: number, temperatureUnit: string, isDaytime: boolean, windSpeed: string, windDirection: string, shortForecast: string, detailedForecast: string, probabilityOfPrecipitation?: { value: number } }} */ period) => ({
      name: period.name,
      temperature: period.temperature,
      temperatureUnit: period.temperatureUnit,
      isDaytime: period.isDaytime,
      windSpeed: period.windSpeed,
      windDirection: period.windDirection,
      shortForecast: period.shortForecast,
      detailedForecast: period.detailedForecast,
      precipitationProbability: period.probabilityOfPrecipitation?.value || 0
    }));
    
    const result = {
      message: days === 1 
        ? `Current weather forecast for ${displayName}. Provide a clear, concise summary of the weather conditions.`
        : `${days}-day weather forecast for ${displayName}. Provide a helpful summary of the weather conditions over the forecast period.`,
      location: displayName,
      coordinates: { lat, lon },
      days: days,
      periodsReturned: formattedPeriods.length,
      forecast: formattedPeriods
    };
    
    console.log('🌤️ Weather tool response:', JSON.stringify(result, null, 2));
    
    return JSON.stringify(result);
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("Error fetching weather data:", error);
    return JSON.stringify({
      error: "Failed to fetch weather data",
      message: errorMessage,
      details: "Please check the location format and try again. Use 'City, State' format for US locations."
    });
  }
}
