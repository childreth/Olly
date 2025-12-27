# Calendar Plugin Documentation

## Overview

The **tauri-plugin-calendar** is a custom Tauri plugin that provides macOS Calendar (EventKit) integration for the Olly application. It enables reading calendar events and generating AI-powered summaries of upcoming appointments.

**Created:** 2025-01-26
**Status:** ✅ Implemented
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
<string>Olly needs access to your calendar to summarize upcoming appointments.</string>
<key>NSCalendarsFullAccessUsageDescription</key>
<string>Olly requires full calendar access to read your upcoming events.</string>
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
```

---

### desktop/macos.rs

Rust FFI interface to Objective-C bridge:

```rust
extern "C" {
    fn calendar_request_permission() -> c_int;
    fn calendar_fetch_events(days_ahead: c_int, json_ptr: *mut *mut c_char) -> c_int;
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

#### `calendar_free_string(ptr)`
- Frees memory allocated by `calendar_fetch_events()`

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
tccutil reset Calendar com.olly.app
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
3. Reset permissions: `tccutil reset Calendar com.olly.app`
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

## Future Enhancements

### Planned Features
- [ ] Create events from chat interface ("Schedule meeting with Bob tomorrow at 3pm")
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

**Last Updated:** 2025-01-26
**Maintainer:** Olly Development Team
**License:** Same as Olly project
