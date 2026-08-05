pub mod cache;
pub mod cli;
pub mod command_line;
pub mod console_app;
pub mod endpoint_monitor;
pub mod file_monitor;
pub mod hidden_command;
pub mod orchestrator;
pub mod port_guard;
pub mod preset_manager;
pub mod process_monitor;
pub mod restart_manager;
pub mod scripting;
pub mod security_audit;
pub mod service_detector;
pub mod smart_filter;
pub mod system_monitor;
pub mod types;
pub mod update_check;

// GUI modules (tray icon on macOS and Windows)
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod app;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod tray_menu;

// Windows-specific: "Start with Windows" registry management
#[cfg(target_os = "windows")]
pub mod autostart;
