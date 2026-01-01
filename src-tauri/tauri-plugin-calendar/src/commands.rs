use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::CalendarExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.calendar().ping(payload)
}

#[command]
pub(crate) async fn request_permission<R: Runtime>(
    app: AppHandle<R>,
) -> Result<PermissionResponse> {
    app.calendar().request_permission()
}

#[command]
pub(crate) async fn check_permission<R: Runtime>(
    app: AppHandle<R>,
) -> Result<String> {
    app.calendar().check_permission()
}

#[command]
pub(crate) async fn fetch_events<R: Runtime>(
    app: AppHandle<R>,
    payload: FetchEventsRequest,
) -> Result<FetchEventsResponse> {
    app.calendar().fetch_events(payload)
}

#[command]
pub(crate) async fn get_diagnostics<R: Runtime>(
    app: AppHandle<R>,
) -> Result<DiagnosticResponse> {
    app.calendar().get_diagnostics()
}

#[command]
pub(crate) async fn create_event<R: Runtime>(
    app: AppHandle<R>,
    payload: CreateEventRequest,
) -> Result<CreateEventResponse> {
    app.calendar().create_event(payload)
}
