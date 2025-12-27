# tauri-plugin-calendar

A Tauri plugin for accessing Apple Calendar (EventKit) on macOS.

## Quick Start

### From Frontend (JavaScript/Svelte)

```javascript
import { invoke } from "@tauri-apps/api/core";

// Request permission
const response = await invoke("plugin:calendar|request_permission");
if (response.granted) {
  // Fetch events for next 14 days
  const { events } = await invoke("plugin:calendar|fetch_events", {
    payload: { daysAhead: 14 }
  });

  console.log(`Found ${events.length} events`);
}
```

### Event Structure

```typescript
{
  title: string;
  startDate: string;      // ISO8601 format
  endDate: string;        // ISO8601 format
  location?: string;
  notes?: string;
  isAllDay: boolean;
  calendarTitle?: string;
}
```

## Commands

### `request_permission`
Requests calendar access permission from the user.

**Returns:** `{ granted: boolean, message?: string }`

### `fetch_events`
Fetches calendar events.

**Parameters:** `{ daysAhead: number }`
**Returns:** `{ events: CalendarEvent[] }`

## Requirements

- **Platform:** macOS only
- **Framework:** EventKit (linked automatically)
- **Permissions:** Calendar entitlement must be configured

## Configuration

Add to parent app's `tauri.conf.json`:

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

Create `Entitlements.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.personal-information.calendars</key>
    <true/>
</dict>
</plist>
```

Create `Info.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSCalendarsUsageDescription</key>
    <string>This app needs access to your calendar to show upcoming events.</string>
</dict>
</plist>
```

## Installation

Add to your app's `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-calendar = { path = "tauri-plugin-calendar" }
```

Register in `main.rs`:

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_calendar::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Architecture

```
Frontend (invoke)
    ↓
Rust Plugin (commands.rs)
    ↓
Desktop/macOS Module (desktop/macos.rs)
    ↓
Objective-C Bridge (macos/calendar_bridge.m)
    ↓
EventKit Framework
```

## File Structure

```
tauri-plugin-calendar/
├── Cargo.toml              # Plugin dependencies
├── build.rs                # Compiles Objective-C bridge
├── README.md               # This file
├── macos/
│   └── calendar_bridge.m   # Objective-C EventKit integration
└── src/
    ├── lib.rs              # Plugin initialization
    ├── commands.rs         # Tauri commands
    ├── models.rs           # Data structures
    ├── error.rs            # Error types
    ├── desktop.rs          # Desktop implementation
    └── desktop/
        └── macos.rs        # macOS FFI bridge
```

## Development

### Build

```bash
cargo build
```

### Test

```bash
# Check compilation
cargo check

# Run with logs
RUST_LOG=debug cargo run
```

## Error Handling

All commands return `Result<T, String>` where errors are human-readable strings:

- "Calendar access is only supported on macOS" - Running on non-macOS
- "Calendar access denied" - User denied permission
- "Failed to fetch calendar events" - EventKit error
- "Invalid UTF-8 from Objective-C" - Encoding issue

## Troubleshooting

### Permission Denied

1. Check `Info.plist` has `NSCalendarsUsageDescription`
2. Check `Entitlements.plist` has calendar entitlement
3. Reset permissions: `tccutil reset Calendar com.your.app`

### Build Failures

1. Ensure Xcode Command Line Tools: `xcode-select --install`
2. Verify EventKit in `tauri.conf.json` frameworks array
3. Check Objective-C syntax in `calendar_bridge.m`

## Documentation

See [PLUGIN_CALENDAR.md](../../PLUGIN_CALENDAR.md) in the project root for comprehensive documentation including:
- Complete architecture diagrams
- Detailed API reference
- Security considerations
- Testing procedures
- Future enhancements

## License

Same as parent Olly project.
