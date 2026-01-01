use crate::models::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

// External functions implemented in Objective-C
extern "C" {
    fn calendar_request_permission() -> c_int;
    fn calendar_check_permission() -> c_int;
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
    fn calendar_get_diagnostics(json_ptr: *mut *mut c_char) -> c_int;
    fn calendar_free_string(ptr: *mut c_char);
}

pub fn get_calendar_diagnostics() -> crate::Result<DiagnosticResponse> {
    unsafe {
        let mut json_ptr: *mut c_char = std::ptr::null_mut();
        let result = calendar_get_diagnostics(&mut json_ptr);

        if result != 0 || json_ptr.is_null() {
            return Err(crate::Error::String("Failed to gather calendar diagnostics".into()));
        }

        let json_str = CStr::from_ptr(json_ptr).to_str()
            .map_err(|e| crate::Error::String(format!("Invalid UTF-8 from Objective-C: {}", e)))?;

        let response: DiagnosticResponse = serde_json::from_str(json_str)
            .map_err(|e| crate::Error::String(format!("Failed to parse diagnostics JSON: {}", e)))?;

        calendar_free_string(json_ptr);

        Ok(response)
    }
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

pub fn create_calendar_event(request: CreateEventRequest) -> crate::Result<CreateEventResponse> {
    use std::ffi::CString;
    
    unsafe {
        // Convert Rust strings to C strings
        let title = CString::new(request.title)
            .map_err(|e| crate::Error::String(format!("Invalid title: {}", e)))?;
        let start_date = CString::new(request.start_date)
            .map_err(|e| crate::Error::String(format!("Invalid start date: {}", e)))?;
        let end_date = CString::new(request.end_date)
            .map_err(|e| crate::Error::String(format!("Invalid end date: {}", e)))?;
        
        let location = request.location.as_ref()
            .map(|s| CString::new(s.as_str()))
            .transpose()
            .map_err(|e| crate::Error::String(format!("Invalid location: {}", e)))?;
        
        let notes = request.notes.as_ref()
            .map(|s| CString::new(s.as_str()))
            .transpose()
            .map_err(|e| crate::Error::String(format!("Invalid notes: {}", e)))?;
        
        let is_all_day = request.is_all_day.unwrap_or(false);
        
        let mut result_json_ptr: *mut c_char = std::ptr::null_mut();
        
        let result = calendar_create_event(
            title.as_ptr(),
            start_date.as_ptr(),
            end_date.as_ptr(),
            location.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            notes.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            if is_all_day { 1 } else { 0 },
            &mut result_json_ptr,
        );
        
        if result_json_ptr.is_null() {
            return Err(crate::Error::String("Failed to create calendar event: no response".into()));
        }
        
        let json_str = CStr::from_ptr(result_json_ptr).to_str()
            .map_err(|e| crate::Error::String(format!("Invalid UTF-8 from Objective-C: {}", e)))?;
        
        let response: CreateEventResponse = serde_json::from_str(json_str)
            .map_err(|e| crate::Error::String(format!("Failed to parse create event response: {}", e)))?;
        
        calendar_free_string(result_json_ptr);
        
        if result != 0 || !response.success {
            return Err(crate::Error::String(
                response.error.unwrap_or_else(|| "Failed to create event".into())
            ));
        }
        
        Ok(response)
    }
}
