# Calendar Plugin Documentation

## Overview

The **tauri-plugin-calendar** is a custom Tauri plugin that provides macOS Calendar (EventKit) integration for the Olly application. It enables reading calendar events, creating new calendar events, and generating AI-powered summaries of upcoming appointments.

**Created:** 2025-01-26
**Last Updated:** 2026-01-01
**Status:** ✅ Implemented (Read & Write)
**Platform:** macOS only (requires EventKit framework)

---

## Architecture

### Component Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│ Frontend (Svelte)                                            │
│ src/lib/components/CalendarSummary.svelte                   │
│  - User interface for calendar display                      │
│  - Permission request UI                                     │
│  - Event list rendering                                      │
└───────────────────┬─────────────────────────────────────────┘
                    │ invoke("plugin:calendar|*")
                    ↓
┌─────────────────────────────────────────────────────────────┐
│ Tauri Plugin (Rust)                                          │
│ src-tauri/tauri-plugin-calendar/                            │
│  - src/lib.rs          → Plugin initialization              │
│  - src/commands.rs     → Command definitions                │
│  - src/models.rs       → Data structures (Events, etc.)     │
│  - src/desktop.rs      → Desktop-specific logic             │
│  - src/desktop/macos.rs → macOS FFI bridge                  │
└───────────────────┬─────────────────────────────────────────┘
                    │ extern "C" FFI calls
                    ↓
┌─────────────────────────────────────────────────────────────┐
│ Objective-C Bridge                                           │
│ macos/calendar_bridge.m                                     │
│  - calendar_request_permission()                            │
│  - calendar_fetch_events()                                  │
│  - calendar_create_event()                                  │
│  - calendar_get_diagnostics()                               │
│  - calendar_free_string()                                   │
└───────────────────┬─────────────────────────────────────────┘
                    │ Native macOS API calls
                    ↓
┌─────────────────────────────────────────────────────────────┐
│ macOS EventKit Framework                                     │
│  - EKEventStore (calendar database)                         │
│  - EKEvent (calendar events)                                │
│  - Authorization management                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
Olly/
├── src-tauri/
│   ├── Info.plist                          # macOS permissions declarations
│   ├── Entitlements.plist                  # Calendar entitlements
│   ├── tauri.conf.json                     # Tauri config (updated)
│   ├── Cargo.toml                          # Main app dependencies (includes plugin)
│   ├── src/
│   │   └── main.rs                         # Main app (includes summarize_calendar_events)
│   └── tauri-plugin-calendar/              # Plugin directory
│       ├── Cargo.toml                      # Plugin dependencies
│       ├── build.rs                        # Build script (compiles Objective-C)
│       ├── macos/
│       │   └── calendar_bridge.m           # Objective-C EventKit bridge
│       └── src/
│           ├── lib.rs                      # Plugin initialization
│           ├── commands.rs                 # Tauri commands
│           ├── models.rs                   # Data structures
│           ├── desktop.rs                  # Desktop implementation
│           └── desktop/
│               └── macos.rs                # macOS FFI interface
└── src/
    ├── routes/
    │   └── +page.svelte                    # Main app UI (includes CalendarSummary)
    └── lib/
        └── components/
            └── CalendarSummary.svelte      # Calendar UI component
```

---

## Data Flow

### 1. Permission Request Flow

```
User clicks "Grant Calendar Access"
    ↓
CalendarSummary.svelte → invoke("plugin:calendar|request_permission")
    ↓
commands.rs → request_permission()
    ↓
desktop/macos.rs → calendar_request_permission() [FFI]
    ↓
calendar_bridge.m → EKEventStore.requestFullAccessToEvents()
    ↓
macOS Permission Prompt shown to user
    ↓
Response: { granted: true/false, message: "..." }
    ↓
CalendarSummary.svelte updates UI state
```

### 2. Event Fetching Flow

```
User granted permission / Auto-refresh
    ↓
CalendarSummary.svelte → invoke("plugin:calendar|fetch_events", { daysAhead: 14 })
    ↓
commands.rs → fetch_events()
    ↓
desktop/macos.rs → calendar_fetch_events(14) [FFI]
    ↓
calendar_bridge.m → EKEventStore.eventsMatchingPredicate()
    ↓
Events converted to JSON: { events: [{ title, startDate, endDate, ... }] }
    ↓
CalendarSummary.svelte receives event array
    ↓
invoke("summarize_calendar_events", { eventsJson, model })
    ↓
main.rs → summarize_calendar_events() → Ollama API
    ↓
AI-generated summary returned to UI
```

### 3. Event Creation Flow

```
User asks AI to create event (e.g., "Schedule meeting tomorrow at 3pm")
    ↓
AI detects intent and calls createCalendarEvent tool
    ↓
tools.js → invoke("plugin:calendar|create_event", { payload: { title, startDate, endDate, ... } })
    ↓
commands.rs → create_event()
    ↓
desktop/macos.rs → calendar_create_event() [FFI]
    ↓
calendar_bridge.m → EKEvent.eventWithEventStore() → store.saveEvent()
    ↓
Response: { success: true, eventId: "...", calendarTitle: "..." }
    ↓
AI confirms event creation to user
```

---

## API Reference

### Plugin Commands (Rust)

#### `request_permission`
**Path:** `plugin:calendar|request_permission`
**Parameters:** None
**Returns:** `PermissionResponse { granted: bool, message: Option<String> }`

Requests calendar access permission from the user. Triggers macOS system permission prompt if not already granted.

**Example:**
```javascript
const response = await invoke("plugin:calendar|request_permission");
if (response.granted) {
  console.log("Calendar access granted!");
}
```

---

#### `fetch_events`
**Path:** `plugin:calendar|fetch_events`
**Parameters:** `FetchEventsRequest { daysAhead: i32 }`
**Returns:** `FetchEventsResponse { events: Vec<CalendarEvent> }`

Fetches calendar events from now to `daysAhead` days in the future.

**CalendarEvent Structure:**
```typescript
{
  title: string,
  startDate: string,        // ISO8601 format
  endDate: string,          // ISO8601 format
  location: string | null,
  notes: string | null,
  isAllDay: boolean,
  calendarTitle: string | null
}
```

**Example:**
```javascript
const response = await invoke("plugin:calendar|fetch_events", {
  payload: { daysAhead: 14 }
});
console.log(`Found ${response.events.length} events`);
```

---

#### `create_event`
**Path:** `plugin:calendar|create_event`
**Parameters:** `CreateEventRequest { title, startDate, endDate, location?, notes?, isAllDay? }`
**Returns:** `CreateEventResponse { success: bool, eventId: Option<String>, calendarTitle: Option<String>, error: Option<String> }`

Creates a new calendar event in the user's default calendar. Uses main-thread affinity and automatic retry logic for ACL errors.

**Robust Fallback Strategy:**
1. Attempts to save to the default calendar.
2. If default is read-only, searches for *any* writable calendar (Local, iCloud, Exchange).
3. If no writable calendars exist, attempts to create a dedicated "Olly" calendar in the user's Local or iCloud account and saves the event there.

---

#### `get_diagnostics`
**Path:** `plugin:calendar|get_diagnostics`
**Parameters:** None
**Returns:** `DiagnosticResponse { authStatus: i32, calendars: Vec<CalendarDiagnostic>, defaultCalendar: Option<String> }`

Gathers detailed information about calendar permissions and available calendars to troubleshoot ACL issues.

**CalendarDiagnostic Structure:**
```typescript
{
  title: string,
  typeCode: number,
  allowsContentModifications: boolean,
  sourceTitle: string,
  isDefault: boolean
}
```

**CreateEventRequest Structure:**
```typescript
{
  title: string,              // Event title (required)
  startDate: string,          // ISO8601 format (required)
  endDate: string,            // ISO8601 format (required)
  location: string | null,    // Optional location
  notes: string | null,       // Optional notes/description
  isAllDay: boolean           // Optional, default false
}
```

**CreateEventResponse Structure:**
```typescript
{
  success: boolean,
  eventId: string | null,           // EventKit identifier if successful
  calendarTitle: string | null,     // Name of calendar where event was created
  error: string | null              // Error message if failed
}
```

**Example:**
```javascript
const response = await invoke("plugin:calendar|create_event", {
  payload: {
    title: "Team Meeting",
    startDate: "2026-01-15T14:00:00-05:00",
    endDate: "2026-01-15T15:00:00-05:00",
    location: "Conference Room A",
    notes: "Discuss Q1 planning",
    isAllDay: false
  }
});

if (response.success) {
  console.log(`Event created: ${response.eventId}`);
}
```

---

### Main App Commands (Rust)

#### `summarize_calendar_events`
**Path:** `summarize_calendar_events` (main app command)
**Parameters:**
- `eventsJson: String` - JSON array of CalendarEvent objects
- `model: Option<String>` - Ollama model name (default: "gemma2:2b")

**Returns:** `String` - AI-generated summary

Uses local Ollama model to generate a natural language summary of calendar events.

**Example:**
```javascript
const summary = await invoke("summarize_calendar_events", {
  eventsJson: JSON.stringify(events),
  model: "gemma2:2b"
});
```

---

## Configuration

### Required Files

#### 1. `Info.plist`
Declares calendar usage purpose to macOS.

```xml
<key>NSCalendarsUsageDescription</key>
<string>Olly needs access to your calendar to summarize upcoming appointments and help you stay organized.</string>
<key>NSCalendarsFullAccessUsageDescription</key>
<string>Olly requires full calendar access to read your upcoming events and create new calendar appointments on your behalf.</string>
<key>NSCalendarsWriteOnlyAccessUsageDescription</key>
<string>Olly needs permission to create calendar events on your behalf.</string>
```

#### 2. `Entitlements.plist`
Grants calendar access entitlement.

```xml
<key>com.apple.security.personal-information.calendars</key>
<true/>
```

#### 3. `tauri.conf.json`
Links EventKit framework and entitlements.

```json
{
  "bundle": {
    "macOS": {
      "entitlements": "Entitlements.plist",
      "frameworks": ["EventKit"]
    }
  }
}
```

---

## Rust Code Structure

### models.rs

Defines data structures for calendar events and responses:

```rust
pub struct CalendarEvent {
  pub title: String,
  pub start_date: String,
  pub end_date: String,
  pub location: Option<String>,
  pub notes: Option<String>,
  pub is_all_day: bool,
  pub calendar_title: Option<String>,
}

pub struct FetchEventsRequest {
  pub days_ahead: i32,
}

pub struct FetchEventsResponse {
  pub events: Vec<CalendarEvent>,
}

pub struct PermissionResponse {
  pub granted: bool,
  pub message: Option<String>,
}

pub struct CreateEventRequest {
  pub title: String,
  pub start_date: String,
  pub end_date: String,
  pub location: Option<String>,
  pub notes: Option<String>,
  pub is_all_day: Option<bool>,
}

pub struct CreateEventResponse {
  pub success: bool,
  pub event_id: Option<String>,
  pub calendar_title: Option<String>,
  pub error: Option<String>,
}
```

---

### desktop/macos.rs

Rust FFI interface to Objective-C bridge:

```rust
extern "C" {
    fn calendar_request_permission() -> c_int;
    fn calendar_fetch_events(days_ahead: c_int, json_ptr: *mut *mut c_char) -> c_int;
    fn calendar_create_event(
        title: *const c_char,
        start_date_iso: *const c_char,
        end_date_iso: *const c_char,
        location: *const c_char,
        notes: *const c_char,
        is_all_day: c_int,
        result_json_ptr: *mut *mut c_char,
    ) -> c_int;
    fn calendar_free_string(ptr: *mut c_char);
}

pub fn request_calendar_permission() -> crate::Result<PermissionResponse> {
    unsafe {
        let result = calendar_request_permission();
        match result {
            1 => Ok(PermissionResponse { granted: true, ... }),
            0 => Ok(PermissionResponse { granted: false, ... }),
            _ => Err(...),
        }
    }
}
```

---

### Objective-C Bridge (calendar_bridge.m)

#### `calendar_request_permission()`
- Checks macOS version (14.0+ uses `requestFullAccessToEvents`)
- Uses semaphore to wait for async permission result
- Returns `1` (granted), `0` (denied), or `-1` (error)

#### `calendar_fetch_events(days_ahead, json_ptr)`
- Checks authorization status
- Creates date range predicate
- Fetches events using `EKEventStore.eventsMatchingPredicate()`
- Converts events to JSON using `NSJSONSerialization`
- Allocates C string via `strdup()` and returns pointer
- Returns `0` (success) or `-1` (error)

#### `calendar_create_event(title, start_date_iso, end_date_iso, location, notes, is_all_day, result_json_ptr)`
- Checks authorization status (accepts Full Access or Write-Only)
- Creates new `EKEvent` with provided details
- Parses ISO8601 date strings using `NSISO8601DateFormatter`
- Sets event to default calendar via `defaultCalendarForNewEvents`
- Saves event using `EKEventStore.saveEvent()`
- Returns JSON response with success status, event ID, and calendar title
- Returns `0` (success) or `-1` (error)

#### `calendar_free_string(ptr)`
- Frees memory allocated by `calendar_fetch_events()` and `calendar_create_event()`

---

## Frontend Component

### CalendarSummary.svelte

**Features:**
- Collapsible widget (▶/▼ toggle)
- Permission request button
- Auto-load on mount if permission granted
- Event list with formatted dates/locations
- AI-generated summary section
- Refresh button
- Error handling with retry

**State Management:**
```javascript
let permissionGranted = false;
let events = [];
let summary = "";
let loading = false;
let error = "";
let isExpanded = false;
```

**Styling:**
- Uses CSS custom properties for theming
- Responsive event cards with color-coded borders
- Loading/error states
- Dark mode support via `var(--surface-color)`

---

## Build Process

### build.rs

The plugin's `build.rs` compiles the Objective-C bridge during Rust build:

```rust
#[cfg(target_os = "macos")]
{
  cc::Build::new()
    .file("macos/calendar_bridge.m")
    .flag("-fmodules")
    .flag("-fobjc-arc")  // Enable ARC for memory management
    .compile("calendar_bridge");

  println!("cargo:rustc-link-lib=framework=EventKit");
  println!("cargo:rustc-link-lib=framework=Foundation");
}
```

**Dependencies:**
- `cc = "1.0"` in `[build-dependencies]`
- Automatically links EventKit and Foundation frameworks

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Calendar access is only supported on macOS" | Running on non-macOS platform | Plugin is macOS-only |
| "Calendar access denied" | User denied permission | Ask user to grant in System Settings > Privacy & Security > Calendars |
| "Failed to fetch calendar events" | Permission not granted or EventKit error | Check authorization status |
| "Failed to call Ollama API" | Ollama not running | Start Ollama: `ollama serve` |
| "Invalid UTF-8 from Objective-C" | Encoding issue in JSON | Check calendar data for special characters |

---

## Testing Checklist

### Manual Testing Steps

- [ ] Permission prompt appears on first launch
- [ ] Permission dialog shows correct app name and reason
- [ ] Granting permission loads events successfully
- [ ] Denying permission shows error message
- [ ] Events display with correct titles, dates, locations
- [ ] All-day events marked with badge
- [ ] Multiple calendars distinguished by color/name
- [ ] AI summary is accurate and concise
- [ ] Refresh button updates events
- [ ] Empty calendar shows friendly message
- [ ] Widget collapses/expands correctly
- [ ] Works with dark/light theme

### Debug Commands

```bash
# Check if plugin compiles
cd src-tauri
cargo check --package tauri-plugin-calendar

# View EventKit framework linking
otool -L target/debug/olly

# Check entitlements in built app
codesign -d --entitlements - Olly.app

# View calendar permissions
tccutil reset Calendar com.olly.chat
```

---

## Troubleshooting

### Plugin Fails to Build

**Symptom:** Compilation errors in `calendar_bridge.m`

**Solutions:**
1. Ensure Xcode Command Line Tools installed: `xcode-select --install`
2. Check EventKit framework linked in `tauri.conf.json`
3. Verify Objective-C syntax in `calendar_bridge.m`

---

### Permission Always Denied

**Symptom:** User grants permission but `granted: false` returned

**Solutions:**
1. Check Info.plist contains `NSCalendarsUsageDescription`
2. Verify Entitlements.plist has calendar entitlement
3. Reset permissions: `tccutil reset Calendar com.olly.chat`
4. Check macOS version (14.0+ requires `requestFullAccessToEvents`)

---

### Events Not Appearing

**Symptom:** Permission granted but events array empty

**Solutions:**
1. Check user has events in Apple Calendar app
2. Verify date range predicate (14 days ahead)
3. Test with different calendars (work, personal, etc.)
4. Check EventKit authorization status in Objective-C

---

## AI Tool Integration

### createCalendarEvent Tool

The calendar event creation is exposed as an AI tool in `src/lib/tools.js`, allowing AI assistants to create calendar events based on natural language requests.

**Tool Definition:**
```javascript
{
  type: 'function',
  function: {
    name: 'createCalendarEvent',
    description: 'Create a new calendar event in the user\'s macOS Calendar...',
    parameters: {
      type: 'object',
      required: ['title', 'startDate', 'endDate'],
      properties: {
        title: { type: 'string', description: 'Event title' },
        startDate: { type: 'string', description: 'ISO8601 format with timezone' },
        endDate: { type: 'string', description: 'ISO8601 format with timezone' },
        location: { type: 'string', description: 'Optional location' },
        notes: { type: 'string', description: 'Optional notes' },
        isAllDay: { type: 'boolean', description: 'All-day event flag' }
      }
    }
  }
}
```

**Supported Models:**
The tool is available when using tool-capable models including:
- Llama 3.1+
- Qwen 2.5+
- Mistral family
- DeepSeek (all versions)
- Command R
- And others (see `supportsToolCalling()` in tools.js)

**Example User Requests:**
- "Schedule a meeting with Bob tomorrow at 3pm"
- "Create a dentist appointment next Tuesday at 10am"
- "Add team standup to my calendar every Monday at 9am"

---

## Future Enhancements

### Planned Features
- [x] Create events from chat interface ("Schedule meeting with Bob tomorrow at 3pm") ✅ **Implemented**
- [ ] Set reminders via AI assistant
- [ ] Conflict detection for scheduling
- [ ] Filter by calendar (e.g., work calendar only)
- [ ] Custom date ranges (not just 2 weeks)
- [ ] Event notifications before appointments
- [ ] Sync calendar with AI conversation context

### Technical Debt
- Add unit tests for FFI boundary
- Add integration tests for Objective-C bridge
- Improve error messages with specific codes
- Add logging/telemetry for permission flow
- Support other platforms (Windows/Linux calendars)

---

## Security Considerations

### Data Privacy
- ✅ Calendar data stays local (processed by local Ollama model by default)
- ⚠️ If using Claude API for summarization, calendar data is sent to Anthropic
- ✅ User must explicitly grant permission via macOS system dialog
- ✅ No calendar data stored/cached by Olly

### Permissions
- Uses Apple's standard EventKit permission system
- Follows macOS 14.0+ full access requirements
- Entitlements properly declared in .plist files
- Permission can be revoked in System Settings

---

## References

### Apple Documentation
- [EventKit Framework](https://developer.apple.com/documentation/eventkit)
- [Accessing Calendar Events](https://developer.apple.com/documentation/eventkit/accessing-calendar-using-eventkit-and-eventkitui)
- [App Sandbox Entitlements](https://developer.apple.com/documentation/bundleresources/entitlements)

### Tauri Documentation
- [Plugin Development](https://v2.tauri.app/develop/plugins/)
- [macOS Bundle Configuration](https://v2.tauri.app/distribute/macos-application-bundle/)
- [Permissions](https://v2.tauri.app/security/permissions/)

### Related Code
- Main app: [src-tauri/src/main.rs](src-tauri/src/main.rs#L2167-L2258) (summarize_calendar_events)
- Frontend: [src/lib/components/CalendarSummary.svelte](src/lib/components/CalendarSummary.svelte)
- Plugin: [src-tauri/tauri-plugin-calendar/](src-tauri/tauri-plugin-calendar/)

---

## Changelog

### v0.2.0 (2026-01-01)
- ✨ **Event Creation:** Create calendar events via AI assistant
- ✅ Added `create_event` Tauri command
- ✅ Objective-C bridge function `calendar_create_event()`
- ✅ AI tool integration (`createCalendarEvent` tool)
- ✅ Full calendar access permission support (read + write)
- ✅ ISO8601 date parsing for event times
- ✅ Support for location, notes, and all-day events
- ✅ Error handling for permission and save failures

### v0.1.0 (2025-01-26)
- ✨ Initial implementation
- ✅ macOS EventKit integration via Objective-C FFI
- ✅ Permission request flow
- ✅ Event fetching (14-day window)
- ✅ AI-powered summarization via Ollama
- ✅ Collapsible calendar widget UI
- ✅ Dark mode support
- ✅ Error handling and retry logic

---

## Contributing

When modifying this plugin:

1. **Update this documentation** when adding features
2. **Test on macOS 14.0+** (EventKit API changes)
3. **Verify memory management** in Objective-C (no leaks)
4. **Check FFI boundary** for UTF-8 encoding issues
5. **Update build.rs** if adding new Objective-C files
6. **Run cargo check** before committing

---

## Support

For issues or questions:
- Check [Troubleshooting](#troubleshooting) section above
- Review macOS Console.app for crash logs
- Enable Rust logging: `RUST_LOG=debug npm run tauri dev`
- Check Ollama logs: `ollama logs`

---

**Last Updated:** 2026-01-01
**Maintainer:** Olly Development Team
**License:** Same as Olly project
