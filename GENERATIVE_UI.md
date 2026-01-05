# Generative UI Documentation

## Overview

The **Generative UI System** is a component-based architecture that enables the Olly application to render rich, interactive Svelte components based on AI tool responses. It follows the Vercel AI SDK pattern where tools and components are associated together, allowing the AI assistant to trigger dynamic UI rendering alongside markdown responses.

**Created:** 2025-01-03
**Status:** ✅ Implemented
**Platform:** Cross-platform (Svelte/Tauri)

---

## Architecture

### Component Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│ Frontend (Svelte)                                            │
│ src/routes/+page.svelte                                     │
│  - Chat interface and message handling                      │
│  - Tool execution orchestration                             │
│  - Component detection and rendering                        │
└───────────────────┬─────────────────────────────────────────┘
                    │ processToolResult()
                    ↓
┌─────────────────────────────────────────────────────────────┐
│ Component Registry                                           │
│ src/lib/components/generative/componentRegistry.js          │
│  - Maps tool names to Svelte components                     │
│  - Provides getComponent() and hasComponent() helpers       │
└───────────────────┬─────────────────────────────────────────┘
                    │ getComponent(componentName)
                    ↓
┌─────────────────────────────────────────────────────────────┐
│ Generative Components (Svelte)                              │
│ src/lib/components/generative/                              │
│  - WeatherCard.svelte      → Weather forecast display       │
│  - [Future components...]  → Calendar, Events, etc.         │
└───────────────────┬─────────────────────────────────────────┘
                    │ Component mounted with data props
                    ↓
┌─────────────────────────────────────────────────────────────┐
│ Tool Functions (JavaScript)                                  │
│ src/lib/tools.js                                            │
│  - getWeather()            → Returns weather data           │
│  - getCalendarEvents()     → Returns calendar events        │
│  - createCalendarEvent()   → Creates calendar events        │
│  - [Other tools...]        → Additional functionality       │
└─────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
Olly/
├── src/
│   ├── lib/
│   │   ├── tools.js                                # Tool definitions and executors
│   │   └── components/
│   │       └── generative/                         # Generative UI components
│   │           ├── WeatherCard.svelte              # Weather forecast component
│   │           ├── componentRegistry.js            # Component registry mapping
│   │           └── [Future components...]
│   └── routes/
│       └── +page.svelte                            # Main chat interface
├── GENERATIVE_UI.md                                # This file
└── PLUGIN_CALENDAR.md                              # Calendar plugin docs
```

---

## Data Flow

### 1. Tool Execution Flow

```
User asks: "What's the weather in Boston, MA?"
    ↓
AI model detects 'getWeather' tool call
    ↓
+page.svelte → executeTool('getWeather', { location: 'Boston, MA', days: 1 })
    ↓
tools.js → getWeather() function executes
    ↓
Fetches weather data from weather.gov API
    ↓
Returns JSON with _component field:
{
  "_component": "WeatherCard",
  "location": "Boston, MA",
  "forecast": [...]
}
    ↓
Tool result added to conversation
```

### 2. Component Detection and Rendering Flow

```
Tool result received in +page.svelte
    ↓
processToolResult(result, toolName) called
    ↓
Parses JSON and checks for _component field
    ↓
hasComponent('WeatherCard') → true
    ↓
getComponent('WeatherCard') → Returns WeatherCard.svelte
    ↓
Creates unique DOM container with ID
    ↓
Adds container div to streamedGreeting HTML
    ↓
responseMarked = marked.parse(streamedGreeting)
    ↓
setTimeout() schedules component mount
    ↓
new WeatherCard({
  target: container,
  props: { data: componentData }
})
    ↓
Component renders in chat interface
```

### 3. Complete Agent Loop with Components

```
User message
    ↓
AI generates response + tool calls
    ↓
For each tool call:
  1. Execute tool → get result
  2. Process result → detect component
  3. If component exists:
     - Render component in UI
     - Add to conversation
  4. If no component:
     - Add plain text to conversation
    ↓
AI generates final response with tool context
    ↓
Final response displayed with rendered components
```

---

## API Reference

### Tool Definition Structure

Each tool in `tools.js` follows the OpenAI function calling schema:

```javascript
{
  type: 'function',
  function: {
    name: 'toolName',
    description: 'What this tool does',
    parameters: {
      type: 'object',
      required: ['requiredParam'],
      properties: {
        requiredParam: {
          type: 'string',
          description: 'Parameter description'
        },
        optionalParam: {
          type: 'string',
          description: 'Optional parameter'
        }
      }
    }
  }
}
```

### Component Data Structure

Tools that render components must return JSON with a `_component` field:

```javascript
{
  "_component": "ComponentName",
  "message": "Text to display alongside component",
  "data": { /* component-specific data */ }
}
```

### Component Props Interface

All generative components receive a `data` prop containing the full tool result:

```javascript
export let data = {};
// data contains:
// - _component: string (component name)
// - message: string (optional description)
// - [tool-specific fields]: any
```

---

## Component Development Guide

### Creating a New Generative Component

#### Step 1: Create Component File

Create `src/lib/components/generative/MyComponent.svelte`:

```svelte
<script>
  /** @type {{ field1?: string, field2?: any }} */
  export let data = {};
  
  const { field1 = 'default', field2 = [] } = data;
</script>

<div class="my-component">
  <!-- Component markup -->
</div>

<style scoped>
  .my-component {
    /* Component styles */
  }
</style>
```

**Key Points:**
- Use JSDoc type annotations for props
- Destructure data with defaults
- Use `scoped` styles to avoid conflicts
- Use CSS custom properties from main app (e.g., `var(--primary)`)

#### Step 2: Register in Component Registry

Update `src/lib/components/generative/componentRegistry.js`:

```javascript
import MyComponent from './MyComponent.svelte';

export const componentRegistry = {
  'WeatherCard': WeatherCard,
  'MyComponent': MyComponent,  // Add here
};
```

#### Step 3: Update Tool to Return Component Data

In `src/lib/tools.js`, modify your tool to include `_component`:

```javascript
async function myTool(args) {
  // ... tool logic ...
  
  const result = {
    _component: 'MyComponent',
    message: 'Description for AI',
    field1: 'value1',
    field2: [/* data */]
  };
  
  return JSON.stringify(result);
}
```

#### Step 4: Test Component

1. Build and run: `npm run tauri dev`
2. Trigger tool via chat: "Ask for my component"
3. Verify component renders in chat interface

---

## Existing Components

### WeatherCard

**Purpose:** Display weather forecast data in a card grid layout

**Data Structure:**
```javascript
{
  "_component": "WeatherCard",
  "message": "Weather forecast description",
  "location": "Boston, MA",
  "coordinates": { lat: 42.3601, lon: -71.0589 },
  "days": 1,
  "periodsReturned": 2,
  "forecast": [
    {
      "name": "Today",
      "temperature": 42,
      "temperatureUnit": "F",
      "isDaytime": true,
      "windSpeed": "10 mph",
      "windDirection": "NW",
      "shortForecast": "Partly cloudy",
      "detailedForecast": "...",
      "precipitationProbability": 20
    }
  ]
}
```

**Features:**
- Responsive grid layout (auto-fit columns)
- Weather emoji based on forecast text
- Temperature display with unit
- Wind speed and precipitation info
- Hover effects and smooth transitions
- Dark mode support

**Styling:**
- Uses app CSS variables for theming
- Gradient background for header
- Card-based layout with shadows
- Mobile-responsive breakpoints

---

## Tool Integration

### getWeather Tool

**File:** `src/lib/tools.js`

**Functionality:**
1. Accepts location (e.g., "Boston, MA") and optional days parameter
2. Calls OpenStreetMap Nominatim API to get coordinates
3. Calls weather.gov API to get forecast data
4. Formats periods and returns with `_component: 'WeatherCard'`

**Returns:**
```javascript
{
  "_component": "WeatherCard",
  "message": "...",
  "location": "...",
  "forecast": [...]
}
```

**Supported Locations:** US locations only (weather.gov limitation)

### Other Tools

- **getCalendarEvents:** Fetches calendar events (no component yet)
- **createCalendarEvent:** Creates calendar events (no component yet)
- **checkCalendarStatus:** Checks calendar permissions (no component yet)

---

## Frontend Integration

### Main Chat Component (+page.svelte)

#### Key Functions

**`processToolResult(toolResult, toolName)`**
- Parses tool result JSON
- Checks for `_component` field
- Returns object with component info or empty state

**Tool Execution Loop**
- Executes tool via `executeTool()`
- Processes result via `processToolResult()`
- If component detected:
  - Creates unique DOM container
  - Mounts component with data props
  - Adds to chat UI

#### Component Rendering Code

**Note:** Svelte 5 uses `mount()` instead of `new Component()`. Components are queued in `pendingComponents` and mounted via `mountPendingComponents()` to handle DOM re-renders during streaming.

```javascript
// Queue component for mounting
if (toolResultInfo.hasComponent && toolResultInfo.componentName) {
  const Component = getComponent(toolResultInfo.componentName);
  if (Component) {
    const componentId = `component-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    streamedGreeting += `\n<div id="${componentId}" class="component-container"></div>\n`;
    responseMarked = await Promise.resolve(marked.parse(streamedGreeting));
    
    // Add to pending components queue for mounting after DOM updates
    pendingComponents.push({
      id: componentId,
      component: Component,
      data: toolResultInfo.componentData
    });
    
    // Mount immediately after this update
    await mountPendingComponents();
  }
}
```

#### Component Mounting (Svelte 5)

The `mountPendingComponents()` function handles mounting with Svelte 5's `mount()` API:

```javascript
import { mount, tick } from "svelte";

async function mountPendingComponents() {
  if (pendingComponents.length === 0) return;
  
  await tick();
  
  for (const pending of pendingComponents) {
    const container = document.getElementById(pending.id);
    if (container && !container.hasChildNodes()) {
      // Svelte 5 uses mount() instead of new Component()
      mount(pending.component, {
        target: container,
        props: { data: pending.data }
      });
    }
  }
}
```

#### Component Persistence

Components persist across follow-up messages because:
1. `pendingComponents` array is **not cleared** between messages
2. `mountPendingComponents()` is called after every `responseMarked` update
3. The `!container.hasChildNodes()` check prevents duplicate mounting

---

## Configuration

### System Prompt

The system prompt in `+page.svelte` instructs the AI about available tools:

```javascript
const systemMsg = `You are Olly, a helpful AI assistant. You have access to external tools for checking calendar status, creating events, fetching events, and checking weather. Always return markdown formatted responses. You like to use emojis when appropriate.`;
```

### Tool Calling Support

Models that support tool calling are detected via `supportsToolCalling()`:

```javascript
const supportedPatterns = [
  /llama-?3\.[1-9]/i,      // Llama 3.1+
  /qwen-?2\.[5-9]/i,       // Qwen 2.5+
  /mistral/i,              // Mistral family
  /command-?r/i,           // Command R
  // ... more patterns
];
```

---

## Styling and Theming

### CSS Custom Properties

Components use app-wide CSS variables:

```css
--primary: Main text color
--secondary: Secondary text color
--tertiary: Tertiary/muted color
--surface-1: Primary background
--surface-2: Secondary background
--surface-3: Tertiary background
--borderRadiusXS: 0.5rem
--borderRadiusSM: 0.75rem
--fontSizeSmall: 0.875rem
--fontSizeMedium: 1rem
--fontSizeLarge: 1.25rem
```

### Dark Mode

Components automatically adapt to dark mode via CSS variables:

```css
[data-theme="dark"] {
  --primary: hsl(var(--hue), 28%, 90%);
  --surface-1: hsl(var(--hue), 70%, 2%);
  /* ... */
}
```

---

## Error Handling

### Tool Execution Errors

If a tool fails:
1. Error is caught in try-catch block
2. Error message added to conversation as tool result
3. AI can provide context-aware error message to user

### Component Rendering Errors

If component fails to mount:
1. Container div still added to UI
2. Component constructor wrapped in try-catch
3. Error logged to console
4. Chat continues without component

### Missing Components

If tool returns unknown component:
1. `hasComponent()` returns false
2. Component rendering skipped
3. Tool result added as plain text to conversation
4. AI responds based on text result

---

## Testing

### Manual Testing Steps

- [ ] Build app: `npm run tauri dev`
- [ ] Ask for weather: "What's the weather in Boston, MA?"
- [ ] Verify WeatherCard component renders
- [ ] Check responsive layout on mobile
- [ ] Test dark/light mode switching
- [ ] Verify component data displays correctly
- [ ] Test with different locations
- [ ] Check console for component mounting logs

### Debug Logging

Enable detailed logging in browser console:

```javascript
// In +page.svelte, logs show:
🔧 Tool calling ENABLED for model: llama2
🔧 Available tools: [...]
🔧 Executing tool: getWeather
✅ Tool getWeather result: {...}
🎨 Component detected: WeatherCard
🎨 Rendering component: WeatherCard
```

---

## Performance Considerations

### Component Mounting

- Components mounted asynchronously via `setTimeout(..., 0)`
- Prevents blocking chat UI during component creation
- Allows DOM to update before component initialization

### Memory Management

- Each component instance is independent
- Old components remain in DOM (part of chat history)
- No automatic cleanup (intentional for conversation history)

### Rendering Optimization

- Use `scoped` styles to prevent CSS conflicts
- Avoid heavy computations in component render
- Use reactive declarations (`$:`) for derived state

---

## Future Enhancements

### Planned Components

- [ ] **CalendarList** - Display upcoming calendar events
- [ ] **EventCard** - Single event detail view
- [ ] **LocationMap** - Show location on map
- [ ] **ChartComponent** - Display data visualizations
- [ ] **CodeBlock** - Syntax-highlighted code display
- [ ] **TableComponent** - Formatted data tables

### Planned Features

- [ ] Component lifecycle hooks (onMount, onDestroy)
- [ ] Component-to-component communication
- [ ] Component state persistence
- [ ] Interactive components (buttons, forms)
- [ ] Component animations and transitions
- [ ] Accessibility improvements (ARIA labels)

### Technical Improvements

- [ ] TypeScript support for component props
- [ ] Component testing framework
- [ ] Component documentation generator
- [ ] Performance profiling tools
- [ ] Component library/showcase

---

## Best Practices

### For Tool Developers

1. **Always include `_component` field** if you want rich UI rendering
2. **Provide meaningful `message` field** for AI context
3. **Include all data needed** by component in result
4. **Handle errors gracefully** with descriptive messages
5. **Test with different data** to ensure component robustness
6. **Document data structure** in tool description

### For Component Developers

1. **Use JSDoc type annotations** for prop validation
2. **Provide sensible defaults** for all props
3. **Use CSS custom properties** for theming consistency
4. **Support dark mode** via CSS variables
5. **Make components responsive** for mobile
6. **Add accessibility features** (ARIA labels, semantic HTML)
7. **Keep components focused** on single responsibility
8. **Test with edge cases** (empty data, long text, etc.)

### For Integration

1. **Register components early** in componentRegistry.js
2. **Update system prompt** when adding new tools
3. **Test tool + component together** end-to-end
4. **Monitor console logs** during development
5. **Verify component mounting** in chat interface
6. **Check responsive behavior** on different screen sizes

---

## Troubleshooting

### Component Not Rendering

**Symptom:** Tool executes but component doesn't appear

**Solutions:**
1. Check browser console for errors
2. Verify component registered in componentRegistry.js
3. Confirm tool returns `_component` field with correct name
4. Check component file exists in `src/lib/components/generative/`
5. Verify component is valid Svelte syntax

### Component Styling Issues

**Symptom:** Component appears but styling is broken

**Solutions:**
1. Check CSS custom properties are defined in app
2. Verify `scoped` attribute on `<style>` tag
3. Check for CSS conflicts with other components
4. Test in both light and dark modes
5. Verify responsive breakpoints work

### Tool Not Calling Component

**Symptom:** Tool executes but returns plain text instead of component

**Solutions:**
1. Verify tool includes `_component` field in result
2. Check `_component` value matches registered component name
3. Confirm tool returns valid JSON
4. Check tool result is being parsed correctly
5. Verify `processToolResult()` is being called

### Component Data Not Displaying

**Symptom:** Component renders but shows empty/wrong data

**Solutions:**
1. Check data structure matches component expectations
2. Verify tool returns all required fields
3. Check component destructuring in `<script>` block
4. Test with hardcoded data to isolate issue
5. Log component props to console for debugging

---

## Security Considerations

### Data Privacy

- ✅ Component data stays local (not sent to external APIs)
- ✅ User can see all data passed to components
- ✅ No sensitive data stored in component state
- ⚠️ Weather data comes from weather.gov (US government)

### Component Isolation

- ✅ Components run in same context as main app
- ✅ No sandboxing between components
- ⚠️ Malicious component could access app state
- Mitigation: Only add trusted components to registry

### XSS Prevention

- ✅ Svelte automatically escapes template expressions
- ✅ Component props are passed as objects, not HTML
- ⚠️ If using `{@html}`, ensure content is sanitized
- Best practice: Avoid `{@html}` in generative components

---

## References

### Vercel AI SDK Pattern

- [Generative UI Documentation](https://sdk.vercel.ai/docs/ai-sdk-ui/generative-user-interfaces)
- [Create UI Components](https://sdk.vercel.ai/docs/ai-sdk-ui/generative-user-interfaces#create-ui-components)

### Svelte Documentation

- [Svelte Components](https://svelte.dev/docs/introduction#components)
- [Reactive Declarations](https://svelte.dev/docs/svelte-components#script-reactive-declarations)
- [Styling](https://svelte.dev/docs/svelte-components#style)

### Related Code

- Tools: [src/lib/tools.js](src/lib/tools.js)
- Chat Interface: [src/routes/+page.svelte](src/routes/+page.svelte)
- Component Registry: [src/lib/components/generative/componentRegistry.js](src/lib/components/generative/componentRegistry.js)
- Weather Component: [src/lib/components/generative/WeatherCard.svelte](src/lib/components/generative/WeatherCard.svelte)

---

## Changelog

### v0.1.1 (2025-01-03)
- 🐛 Fixed Svelte 5 compatibility - use `mount()` instead of `new Component()`
- 🐛 Fixed component persistence across follow-up messages
- 📝 Updated documentation with Svelte 5 mounting code

### v0.1.0 (2025-01-03)
- ✨ Initial implementation
- ✅ Component registry system
- ✅ WeatherCard component
- ✅ Tool result detection and component rendering
- ✅ Dynamic component mounting
- ✅ Dark mode support
- ✅ Responsive design

---

## Contributing

When adding new generative components:

1. **Create component file** in `src/lib/components/generative/`
2. **Add JSDoc type annotations** for props
3. **Register in componentRegistry.js**
4. **Update tool** to return `_component` field
5. **Test end-to-end** with actual tool execution
6. **Update this documentation** with component details
7. **Add to [Planned Components](#planned-components)** if not yet complete

---

## Support

For issues or questions:

- Check [Troubleshooting](#troubleshooting) section above
- Review browser console for error messages
- Check component props match tool result structure
- Verify component is registered in componentRegistry.js
- Test with simpler data first

---

**Last Updated:** 2025-01-03
**Maintainer:** Olly Development Team
**License:** Same as Olly project
