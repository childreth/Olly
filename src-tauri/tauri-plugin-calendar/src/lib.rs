use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Calendar;
#[cfg(mobile)]
use mobile::Calendar;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the calendar APIs.
pub trait CalendarExt<R: Runtime> {
  fn calendar(&self) -> &Calendar<R>;
}

impl<R: Runtime, T: Manager<R>> crate::CalendarExt<R> for T {
  fn calendar(&self) -> &Calendar<R> {
    self.state::<Calendar<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("calendar")
    .invoke_handler(tauri::generate_handler![
      commands::ping,
      commands::request_permission,
      commands::check_permission,
      commands::fetch_events
    ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let calendar = mobile::init(app, api)?;
      #[cfg(desktop)]
      let calendar = desktop::init(app, api)?;
      app.manage(calendar);
      Ok(())
    })
    .build()
}
