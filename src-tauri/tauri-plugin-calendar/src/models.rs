use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingRequest {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
  pub title: String,
  pub start_date: String,
  pub end_date: String,
  pub location: Option<String>,
  pub notes: Option<String>,
  pub is_all_day: bool,
  pub is_recurring: bool,
  pub calendar_title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchEventsRequest {
  pub days_ahead: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchEventsResponse {
  pub events: Vec<CalendarEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionResponse {
  pub granted: bool,
  pub message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventRequest {
  pub title: String,
  pub start_date: String,  // ISO8601 format
  pub end_date: String,    // ISO8601 format
  pub location: Option<String>,
  pub notes: Option<String>,
  pub is_all_day: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventResponse {
  pub success: bool,
  pub event_id: Option<String>,
  pub calendar_title: Option<String>,
  pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDiagnostic {
  pub title: String,
  pub type_code: i32,
  pub allows_content_modifications: bool,
  pub source_title: String,
  pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticResponse {
  pub auth_status: i32,
  pub calendars: Vec<CalendarDiagnostic>,
  pub default_calendar: Option<String>,
}
