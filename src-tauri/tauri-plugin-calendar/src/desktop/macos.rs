use crate::models::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

// External functions implemented in Objective-C
extern "C" {
    fn calendar_request_permission() -> c_int;
    fn calendar_check_permission() -> c_int;
    fn calendar_fetch_events(days_ahead: c_int, json_ptr: *mut *mut c_char) -> c_int;
    fn calendar_free_string(ptr: *mut c_char);
}

pub fn request_calendar_permission() -> crate::Result<PermissionResponse> {
    unsafe {
        let result = calendar_request_permission();

        match result {
            1 => Ok(PermissionResponse {
                granted: true,
                message: Some("Calendar access granted".into()),
            }),
            0 => Ok(PermissionResponse {
                granted: false,
                message: Some("Calendar access denied".into()),
            }),
            _ => Err(crate::Error::String("Failed to request calendar permission".into())),
        }
    }
}

pub fn check_calendar_permission() -> crate::Result<String> {
    unsafe {
        let result = calendar_check_permission();
        match result {
            2 => Ok("authorized".into()),
            1 => Ok("denied".into()),
            0 => Ok("prompt".into()), // Not determined
            _ => Err(crate::Error::String("Failed to check calendar permission".into())),
        }
    }
}

pub fn fetch_calendar_events(days_ahead: i32) -> crate::Result<FetchEventsResponse> {
    unsafe {
        let mut json_ptr: *mut c_char = std::ptr::null_mut();
        let result = calendar_fetch_events(days_ahead as c_int, &mut json_ptr);

        if result != 0 || json_ptr.is_null() {
            return Err(crate::Error::String("Failed to fetch calendar events".into()));
        }

        let json_str = CStr::from_ptr(json_ptr).to_str()
            .map_err(|e| crate::Error::String(format!("Invalid UTF-8 from Objective-C: {}", e)))?;

        let response: FetchEventsResponse = serde_json::from_str(json_str)
            .map_err(|e| crate::Error::String(format!("Failed to parse events JSON: {}", e)))?;

        calendar_free_string(json_ptr);

        Ok(response)
    }
}
