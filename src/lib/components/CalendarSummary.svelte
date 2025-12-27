<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let permissionGranted = false;
  let events = [];
  let summary = "";
  let loading = false;
  let error = "";
  let isExpanded = false;

  onMount(async () => {
    // Check permission status before attempting to load
    try {
      const status = await invoke("plugin:calendar|check_permission");
      console.log("Calendar permission status:", status);
      
      if (status === "authorized") {
        permissionGranted = true;
        await loadEvents(false);
      } else {
        permissionGranted = false;
        console.log("Calendar permission not granted yet (status: " + status + ")");
      }
    } catch (err) {
      console.error("Failed to check calendar permission:", err);
    }
  });

  async function requestPermission() {
    console.log("🔔 requestPermission() called");
    loading = true;
    error = "";
    try {
      console.log("📞 Invoking plugin:calendar|request_permission...");
      const response = await invoke("plugin:calendar|request_permission");
      console.log("✅ Response received:", response);
      permissionGranted = response.granted;

      if (permissionGranted) {
        await loadEvents(false);
      } else {
        error = "Calendar access was denied. Please grant access in System Settings > Privacy & Security > Calendars.";
      }
    } catch (err) {
      error = `Permission error: ${err}`;
      console.error("Permission error:", err);
    }
    loading = false;
  }

  async function loadEvents(silent = false) {
    // Handle case where silent is an Event object (from on:click binding without wrapper)
    const isSilent = typeof silent === 'boolean' ? silent : false;

    loading = true;
    if (!isSilent) error = "";
    try {
      const response = await invoke("plugin:calendar|fetch_events", {
        payload: {
          daysAhead: 14
        }
      });

      events = response.events || [];
      permissionGranted = true;

      if (events.length > 0) {
        await generateSummary();
      } else {
        summary = "You have no upcoming events in the next 2 weeks. Enjoy your free time!";
      }
    } catch (err) {
      if (!isSilent) {
        error = `Failed to load events: ${err}`;
        console.error("Load events error:", err);
      } else {
        console.log("Initial load check (permission likely not granted yet):", err);
      }
      permissionGranted = false;
    }
    loading = false;
  }

  async function generateSummary() {
    try {
      summary = await invoke("summarize_calendar_events", {
        eventsJson: JSON.stringify(events),
        model: "gemma3:1b" // You can make this configurable
      });
    } catch (err) {
      error = `Failed to generate summary: ${err}`;
      console.error("Summary generation error:", err);
    }
  }

  function toggleExpanded() {
    isExpanded = !isExpanded;
  }

  function handleKeydown(event) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggleExpanded();
    }
  }

  function formatDate(isoString) {
    const date = new Date(isoString);
    return date.toLocaleDateString(undefined, {
      weekday: 'short',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
</script>

<div class="calendar-summary">
  <div class="calendar-header" on:click={toggleExpanded} on:keydown={handleKeydown} role="button" tabindex="0">
    <h3>📅 Upcoming Events (Next 2 Weeks)</h3>
    <span class="expand-icon">{isExpanded ? '▼' : '▶'}</span>
  </div>

  {#if isExpanded}
    <div class="calendar-content">
      {#if !permissionGranted}
        <div class="permission-prompt">
          <p>Olly needs permission to access your calendar to show upcoming events.</p>
          <button
            class="grant-button"
            on:click={requestPermission}
            disabled={loading}
          >
            {loading ? 'Requesting...' : 'Grant Calendar Access'}
          </button>
        </div>
      {:else if loading}
        <div class="loading">
          <p>Loading your calendar events...</p>
        </div>
      {:else if error}
        <div class="error">
          <p>{error}</p>
          <button
            class="retry-button"
            on:click={() => loadEvents(false)}
          >
            Retry
          </button>
        </div>
      {:else}
        <div class="summary-section">
          <h4>Summary</h4>
          <p class="summary-text">{summary}</p>
        </div>

        {#if events.length > 0}
          <div class="events-list">
            <h4>Events ({events.length})</h4>
            {#each events as event}
              <div class="event-item">
                <div class="event-title">{event.title}</div>
                <div class="event-details">
                  <span class="event-time">{formatDate(event.startDate)}</span>
                  {#if event.location}
                    <span class="event-location">📍 {event.location}</span>
                  {/if}
                  {#if event.isAllDay}
                    <span class="event-badge">All Day</span>
                  {/if}
                  {#if event.calendarTitle}
                    <span class="event-calendar">[{event.calendarTitle}]</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}

        <div class="actions">
          <button
            class="refresh-button"
            on:click={() => loadEvents(false)}
          >
            Refresh
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .calendar-summary {
    background: var(--surface-color, #1e1e1e);
    border-radius: 8px;
    padding: 16px;
    margin: 16px 0;
    border: 1px solid var(--border-color, #333);
  }

  .calendar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    user-select: none;
  }

  .calendar-header:hover {
    opacity: 0.8;
  }

  .calendar-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .expand-icon {
    font-size: 12px;
    color: var(--text-secondary, #999);
  }

  .calendar-content {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #333);
  }

  .permission-prompt, .loading, .error {
    text-align: center;
    padding: 20px;
    position: relative;
    z-index: 1;
  }

  .permission-prompt :global(#buttonWrap),
  .error :global(#buttonWrap) {
    pointer-events: auto;
    position: relative;
    z-index: 10;
  }

  .error {
    color: #ff6b6b;
  }

  .summary-section {
    margin-bottom: 20px;
    padding: 16px;
    background: var(--background-secondary, #2a2a2a);
    border-radius: 6px;
  }

  .summary-section h4 {
    margin: 0 0 12px 0;
    font-size: 14px;
    text-transform: uppercase;
    color: var(--text-secondary, #999);
    font-weight: 600;
  }

  .summary-text {
    margin: 0;
    line-height: 1.6;
    color: var(--text-primary, #fff);
  }

  .events-list {
    margin-bottom: 20px;
  }

  .events-list h4 {
    margin: 0 0 12px 0;
    font-size: 14px;
    text-transform: uppercase;
    color: var(--text-secondary, #999);
    font-weight: 600;
  }

  .event-item {
    padding: 12px;
    margin-bottom: 8px;
    background: var(--background-secondary, #2a2a2a);
    border-radius: 6px;
    border-left: 3px solid var(--accent-color, #4a9eff);
  }

  .event-title {
    font-weight: 600;
    margin-bottom: 6px;
    color: var(--text-primary, #fff);
  }

  .event-details {
    font-size: 13px;
    color: var(--text-secondary, #999);
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .event-time {
    color: var(--text-secondary, #999);
  }

  .event-location {
    color: var(--text-secondary, #999);
  }

  .event-badge {
    background: var(--accent-color, #4a9eff);
    color: white;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 600;
  }

  .event-calendar {
    color: var(--text-tertiary, #666);
    font-style: italic;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    padding-top: 12px;
  }

  .grant-button,
  .retry-button,
  .refresh-button {
    background-color: var(--buttonbg, #4a9eff);
    color: white;
    border: none;
    padding: 12px 24px;
    font-size: 16px;
    font-weight: 600;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: var(--bodyFamily, system-ui);
  }

  .grant-button:hover,
  .retry-button:hover,
  .refresh-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(74, 158, 255, 0.4);
  }

  .grant-button:active,
  .retry-button:active,
  .refresh-button:active {
    transform: translateY(0);
  }

  .grant-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .grant-button {
    margin-top: 12px;
  }

  .retry-button,
  .refresh-button {
    padding: 10px 20px;
    font-size: 14px;
  }
</style>
