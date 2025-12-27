use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

#[cfg(target_os = "macos")]
mod macos;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Calendar<R>> {
  Ok(Calendar(app.clone()))
}

/// Access to the calendar APIs.
pub struct Calendar<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Calendar<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }

  #[cfg(target_os = "macos")]
  pub fn request_permission(&self) -> crate::Result<PermissionResponse> {
    macos::request_calendar_permission()
  }

  #[cfg(target_os = "macos")]
  pub fn check_permission(&self) -> crate::Result<String> {
    macos::check_calendar_permission()
  }

  #[cfg(not(target_os = "macos"))]
  pub fn request_permission(&self) -> crate::Result<PermissionResponse> {
    Err(crate::Error::String("Calendar access is only supported on macOS".into()))
  }

  #[cfg(not(target_os = "macos"))]
  pub fn check_permission(&self) -> crate::Result<String> {
    Ok("unsupported".into())
  }

  #[cfg(target_os = "macos")]
  pub fn fetch_events(&self, payload: FetchEventsRequest) -> crate::Result<FetchEventsResponse> {
    macos::fetch_calendar_events(payload.days_ahead)
  }

  #[cfg(not(target_os = "macos"))]
  pub fn fetch_events(&self, _payload: FetchEventsRequest) -> crate::Result<FetchEventsResponse> {
    Err(crate::Error::String("Calendar access is only supported on macOS".into()))
  }
}
