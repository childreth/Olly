const COMMANDS: &[&str] = &["ping", "request_permission", "check_permission", "fetch_events"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();

  // Compile Objective-C bridge for macOS
  #[cfg(target_os = "macos")]
  {
    cc::Build::new()
      .file("macos/calendar_bridge.m")
      .flag("-fmodules")
      .flag("-fobjc-arc")
      .flag("-fobjc-exceptions")
      .compile("calendar_bridge");

    println!("cargo:rustc-link-lib=framework=EventKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rerun-if-changed=macos/calendar_bridge.m");
  }
}
