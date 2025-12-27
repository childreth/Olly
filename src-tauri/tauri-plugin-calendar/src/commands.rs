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
