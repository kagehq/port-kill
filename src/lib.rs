pub mod console_app;
pub mod process_monitor;
pub mod types;
pub mod cli;
pub mod smart_filter;
pub mod system_monitor;
pub mod port_guard;
pub mod security_audit;
pub mod endpoint_monitor;
pub mod scripting;

// macOS-specific modules (only compiled on macOS)
#[cfg(target_os = "macos")]
pub mod app;
#[cfg(target_os = "macos")]
pub mod tray_menu;
