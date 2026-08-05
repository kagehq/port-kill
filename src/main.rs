// On Windows, build the tray binary as a GUI-subsystem app so launching it
// from Explorer doesn't open a console window. CLI invocations from a
// terminal re-attach to the parent console (see attach_parent_console).
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
use anyhow::Result;
#[cfg(target_os = "macos")]
use clap::Parser;
#[cfg(target_os = "macos")]
use log::info;
#[cfg(target_os = "macos")]
use port_kill::cache::output::print_or_json;
#[cfg(target_os = "macos")]
use port_kill::cache::{
    clean::clean_caches,
    doctor::doctor,
    list::{list_caches, print_list_table},
    restore::restore_last_backup,
};
#[cfg(target_os = "macos")]
use port_kill::update_check;
#[cfg(target_os = "macos")]
use port_kill::{app::PortKillApp, cli::Args, console_app::ConsolePortKillApp};

#[cfg(target_os = "macos")]
fn main() -> Result<()> {
    // Parse command-line arguments
    let mut args = Args::parse();

    // Handle update check
    if args.check_updates {
        let current_version = env!("CARGO_PKG_VERSION");
        let rt = tokio::runtime::Runtime::new()?;
        match rt.block_on(update_check::check_for_updates(current_version)) {
            Ok(Some(update_info)) => {
                update_check::print_update_check_result(&update_info);
                return Ok(());
            }
            Ok(None) => {
                println!("✅ You're running the latest version ({})", current_version);
                return Ok(());
            }
            Err(e) => {
                eprintln!("⚠️  Could not check for updates: {}", e);
                return Ok(());
            }
        }
    }

    // Determine if this is a quick operation that will exit early
    // Skip update check for these to avoid 1-5+ second network delays
    let is_quick_operation = args.list_presets
        || args.save_preset.is_some()
        || args.delete_preset.is_some()
        || args.cache.is_some();

    // Check for updates only for long-running operations
    if !is_quick_operation {
        let current_version = env!("CARGO_PKG_VERSION");
        let rt = tokio::runtime::Runtime::new()?;
        if let Ok(Some(update_info)) = rt.block_on(update_check::check_for_updates(current_version)) {
            update_check::print_update_notification(&update_info);
        }
    }

    // Handle preset functionality
    if args.list_presets {
        match Args::list_available_presets() {
            Ok(presets_list) => {
                println!("{}", presets_list);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Save preset
    if let Some(name) = args.save_preset.clone() {
        // Validate arguments before building preset to catch malformed port specifications
        if let Err(e) = args.validate() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        let desc = args
            .preset_desc
            .clone()
            .unwrap_or_else(|| "User-defined preset".to_string());
        let preset = args.build_preset_from_args(name.clone(), desc);
        let mut mgr = port_kill::preset_manager::PresetManager::new();
        if let Err(e) = mgr.load_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        mgr.add_preset(preset);
        if let Err(e) = mgr.save_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        println!("✅ Saved preset '{}'.", name);
        return Ok(());
    }

    // Delete preset
    if let Some(name) = args.delete_preset.clone() {
        let mut mgr = port_kill::preset_manager::PresetManager::new();
        if let Err(e) = mgr.load_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        match mgr.remove_preset(&name) {
            Some(_) => {
                if let Err(e) = mgr.save_presets() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                println!("🗑️  Deleted preset '{}'.", name);
            }
            None => {
                eprintln!("Preset '{}' not found.", name);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Apply preset if specified
    if let Some(preset_name) = args.preset.clone() {
        if let Err(e) = args.load_preset(&preset_name) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Set up logging level based on log_level argument
    let log_level = if args.verbose {
        // Verbose flag overrides log_level for backward compatibility
        "debug"
    } else {
        args.log_level.to_rust_log()
    };
    std::env::set_var("RUST_LOG", log_level);

    // Initialize logging
    env_logger::init();

    info!("Starting Port Kill application...");
    info!("Monitoring: {}", args.get_port_description());

    // Handle cache subcommand: route to console-like behavior
    if let Some(cache_cmd) = args.cache.clone() {
        let c = cache_cmd.args();
        if c.list || c.dry_run {
            let resp = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(list_caches(
                    &c.lang,
                    c.npx,
                    c.js_pm,
                    c.hf,
                    c.torch,
                    c.vercel,
                    c.cloudflare,
                    c.stale_days,
                ));
            if c.json {
                print_or_json(&resp, true);
            } else {
                print_list_table(&resp);
            }
            return Ok(());
        }
        if c.clean {
            let resp = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(clean_caches(
                    &c.lang,
                    c.npx,
                    c.js_pm,
                    c.safe_delete,
                    c.force,
                    c.hf,
                    c.torch,
                    c.vercel,
                    c.cloudflare,
                    c.stale_days,
                ));
            print_or_json(&resp, c.json);
            return Ok(());
        }
        if c.restore_last {
            let resp = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(restore_last_backup());
            print_or_json(&resp, c.json);
            if resp.error.is_some() {
                std::process::exit(1);
            }
            return Ok(());
        }
        if c.doctor {
            let report = tokio::runtime::Runtime::new().unwrap().block_on(doctor());
            print_or_json(&report, c.json);
            return Ok(());
        }
    }

    // Handle new lifecycle management features
    // These run in console mode even from the GUI binary
    
    if let Some(port) = args.restart {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.restart_port(port).await
        })?;
        return Ok(());
    }

    if args.show_restart_history {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.show_restart_history().await
        })?;
        return Ok(());
    }

    if let Some(port) = args.clear_restart {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.clear_restart_history(port).await
        })?;
        return Ok(());
    }

    if args.detect {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.detect_services().await
        })?;
        return Ok(());
    }

    if let Some(service_name) = args.start.clone() {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.start_service(&service_name).await
        })?;
        return Ok(());
    }

    if args.init_config {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.init_config().await
        })?;
        return Ok(());
    }

    if args.up {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.orchestrate_up().await
        })?;
        return Ok(());
    }

    if args.down {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.orchestrate_down().await
        })?;
        return Ok(());
    }

    if let Some(service_name) = args.restart_service.clone() {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.orchestrate_restart(&service_name).await
        })?;
        return Ok(());
    }

    if args.status {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.orchestrate_status().await
        })?;
        return Ok(());
    }

    // Create and run the application (GUI mode)
    let app = PortKillApp::new(args)?;
    app.run()?;

    info!("Port Kill application stopped");
    Ok(())
}

#[cfg(target_os = "windows")]
use anyhow::Result;
#[cfg(target_os = "windows")]
use clap::Parser;
#[cfg(target_os = "windows")]
use log::info;
#[cfg(target_os = "windows")]
use port_kill::cache::output::print_or_json;
#[cfg(target_os = "windows")]
use port_kill::cache::{
    clean::clean_caches,
    doctor::doctor,
    list::{list_caches, print_list_table},
    restore::restore_last_backup,
};
#[cfg(target_os = "windows")]
use port_kill::{app::PortKillApp, cli::Args, console_app::ConsolePortKillApp};

/// With the "windows" subsystem there is no console by default. If the app was
/// launched from a terminal, re-attach to it so CLI output (--list, --console,
/// --help, etc.) still works; when launched from Explorer this is a no-op and
/// no console window is created.
#[cfg(target_os = "windows")]
fn attach_parent_console() {
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::{
        AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE,
    };
    unsafe {
        // Only attach when stdout is not already usable: if the caller
        // redirected it (pipe, file, PowerShell capture), attaching would
        // clobber those handles and the output would be lost.
        let stdout = GetStdHandle(STD_OUTPUT_HANDLE);
        if stdout == 0 || stdout == INVALID_HANDLE_VALUE {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

#[cfg(target_os = "windows")]
fn main() -> Result<()> {
    attach_parent_console();

    // Parse command-line arguments
    let mut args = Args::parse();

    // Handle self-update
    if args.self_update {
        let rt = tokio::runtime::Runtime::new()?;
        match rt.block_on(port_kill::update_check::self_update()) {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("⚠️  Self-update failed: {}", e);
                return Ok(());
            }
        }
    }

    // Handle update check
    if args.check_updates {
        let current_version = env!("CARGO_PKG_VERSION");
        let rt = tokio::runtime::Runtime::new()?;
        match rt.block_on(port_kill::update_check::check_for_updates(current_version)) {
            Ok(Some(update_info)) => {
                port_kill::update_check::print_update_check_result(&update_info);
                return Ok(());
            }
            Ok(None) => {
                println!("✅ You're running the latest version ({})", current_version);
                return Ok(());
            }
            Err(e) => {
                eprintln!("⚠️  Could not check for updates: {}", e);
                return Ok(());
            }
        }
    }

    // Determine if this is a quick operation that will exit early
    // Skip update check for these to avoid 1-5+ second network delays
    let is_quick_operation = args.list_presets
        || args.save_preset.is_some()
        || args.delete_preset.is_some()
        || args.cache.is_some();

    // Check for updates only for long-running operations
    if !is_quick_operation {
        let current_version = env!("CARGO_PKG_VERSION");
        let rt = tokio::runtime::Runtime::new()?;
        if let Ok(Some(update_info)) = rt.block_on(port_kill::update_check::check_for_updates(current_version)) {
            port_kill::update_check::print_update_notification(&update_info);
        }
    }

    // Handle preset functionality
    if args.list_presets {
        match Args::list_available_presets() {
            Ok(presets_list) => {
                println!("{}", presets_list);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Save preset
    if let Some(name) = args.save_preset.clone() {
        // Validate arguments before building preset to catch malformed port specifications
        if let Err(e) = args.validate() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        let desc = args
            .preset_desc
            .clone()
            .unwrap_or_else(|| "User-defined preset".to_string());
        let preset = args.build_preset_from_args(name.clone(), desc);
        let mut mgr = port_kill::preset_manager::PresetManager::new();
        if let Err(e) = mgr.load_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        mgr.add_preset(preset);
        if let Err(e) = mgr.save_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        println!("✅ Saved preset '{}'.", name);
        return Ok(());
    }

    // Delete preset
    if let Some(name) = args.delete_preset.clone() {
        let mut mgr = port_kill::preset_manager::PresetManager::new();
        if let Err(e) = mgr.load_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        match mgr.remove_preset(&name) {
            Some(_) => {
                if let Err(e) = mgr.save_presets() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                println!("🗑️  Deleted preset '{}'.", name);
            }
            None => {
                eprintln!("Preset '{}' not found.", name);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Apply preset if specified
    if let Some(preset_name) = args.preset.clone() {
        if let Err(e) = args.load_preset(&preset_name) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Set up logging level based on log_level argument
    let log_level = if args.verbose {
        // Verbose flag overrides log_level for backward compatibility
        "debug"
    } else {
        args.log_level.to_rust_log()
    };
    std::env::set_var("RUST_LOG", log_level);

    // Initialize logging
    env_logger::init();

    info!("Starting Port Kill application on Windows...");
    info!("Monitoring: {}", args.get_port_description());

    // Handle cache subcommand: route to console-like behavior
    if let Some(cache_cmd) = args.cache.clone() {
        let c = cache_cmd.args();
        if c.list || c.dry_run {
            let resp = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(list_caches(
                    &c.lang,
                    c.npx,
                    c.js_pm,
                    c.hf,
                    c.torch,
                    c.vercel,
                    c.cloudflare,
                    c.stale_days,
                ));
            if c.json {
                print_or_json(&resp, true);
            } else {
                print_list_table(&resp);
            }
            return Ok(());
        }
        if c.clean {
            let resp = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(clean_caches(
                    &c.lang,
                    c.npx,
                    c.js_pm,
                    c.safe_delete,
                    c.force,
                    c.hf,
                    c.torch,
                    c.vercel,
                    c.cloudflare,
                    c.stale_days,
                ));
            print_or_json(&resp, c.json);
            return Ok(());
        }
        if c.restore_last {
            let resp = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(restore_last_backup());
            print_or_json(&resp, c.json);
            if resp.error.is_some() {
                std::process::exit(1);
            }
            return Ok(());
        }
        if c.doctor {
            let report = tokio::runtime::Runtime::new().unwrap().block_on(doctor());
            print_or_json(&report, c.json);
            return Ok(());
        }
    }

    // Handle new lifecycle management features
    // These run in console mode even from the GUI binary

    if let Some(port) = args.restart {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.restart_port(port).await
        })?;
        return Ok(());
    }

    if args.show_restart_history {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.show_restart_history().await
        })?;
        return Ok(());
    }

    if let Some(port) = args.clear_restart {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.clear_restart_history(port).await
        })?;
        return Ok(());
    }

    if args.detect {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.detect_services().await
        })?;
        return Ok(());
    }

    if let Some(service_name) = args.start.clone() {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.start_service(&service_name).await
        })?;
        return Ok(());
    }

    if args.init_config {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.init_config().await
        })?;
        return Ok(());
    }

    if args.up {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.orchestrate_up().await
        })?;
        return Ok(());
    }

    if args.down {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.orchestrate_down().await
        })?;
        return Ok(());
    }

    if let Some(service_name) = args.restart_service.clone() {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.orchestrate_restart(&service_name).await
        })?;
        return Ok(());
    }

    if args.status {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.orchestrate_status().await
        })?;
        return Ok(());
    }

    // Route console-style invocations (explicit --console, one-shot operations,
    // scripting, JSON output, etc.) to the console app so CLI usage keeps working.
    // Only a plain monitoring invocation starts the tray UI below.
    let wants_console = args.console
        || args.list
        || !args.positional_ports.is_empty()
        || args.clear.is_some()
        || args.kill.is_some()
        || args.kill_all
        || args.reset
        || args.kill_file.is_some()
        || args.kill_ext.is_some()
        || args.list_file.is_some()
        || args.json
        || args.monitor_endpoint.is_some()
        || args.guard.is_some()
        || args.guard_mode
        || args.audit
        || args.script.is_some()
        || args.script_file.is_some();

    if wants_console {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.run().await
        })?;
        info!("Port Kill application stopped");
        return Ok(());
    }

    // Create and run the application (GUI tray mode, like macOS).
    // If the tray cannot be created (e.g. headless session), fall back to console mode.
    let gui_result = match PortKillApp::new(args.clone()) {
        Ok(app) => app.run(),
        Err(e) => Err(e),
    };

    if let Err(e) = gui_result {
        log::warn!(
            "Tray mode failed on Windows ({}). Falling back to console mode...",
            e
        );
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let app = ConsolePortKillApp::new(args)?;
            app.run().await
        })?;
    }

    info!("Port Kill application stopped");
    Ok(())
}

#[cfg(target_os = "linux")]
use anyhow::Result;
#[cfg(target_os = "linux")]
use clap::Parser;
#[cfg(target_os = "linux")]
use log::info;
#[cfg(target_os = "linux")]
use port_kill::cache::output::print_or_json;
#[cfg(target_os = "linux")]
use port_kill::cache::{
    clean::clean_caches,
    doctor::doctor,
    list::{list_caches, print_list_table},
    restore::restore_last_backup,
};
#[cfg(target_os = "linux")]
use port_kill::{cli::Args, console_app::ConsolePortKillApp};

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let mut args = Args::parse();

    // Handle self-update
    if args.self_update {
        match port_kill::update_check::self_update().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("⚠️  Self-update failed: {}", e);
                return Ok(());
            }
        }
    }

    // Handle update check
    if args.check_updates {
        let current_version = env!("CARGO_PKG_VERSION");
        match port_kill::update_check::check_for_updates(current_version).await {
            Ok(Some(update_info)) => {
                port_kill::update_check::print_update_check_result(&update_info);
                return Ok(());
            }
            Ok(None) => {
                println!("✅ You're running the latest version ({})", current_version);
                return Ok(());
            }
            Err(e) => {
                eprintln!("⚠️  Could not check for updates: {}", e);
                return Ok(());
            }
        }
    }

    // Determine if this is a quick operation that will exit early
    // Skip update check for these to avoid 1-5+ second network delays
    let is_quick_operation = args.list_presets
        || args.save_preset.is_some()
        || args.delete_preset.is_some()
        || args.cache.is_some();

    // Check for updates only for long-running operations
    if !is_quick_operation {
        let current_version = env!("CARGO_PKG_VERSION");
        if let Ok(Some(update_info)) = port_kill::update_check::check_for_updates(current_version).await {
            port_kill::update_check::print_update_notification(&update_info);
        }
    }

    // Handle preset functionality
    if args.list_presets {
        match Args::list_available_presets() {
            Ok(presets_list) => {
                println!("{}", presets_list);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Save preset
    if let Some(name) = args.save_preset.clone() {
        // Validate arguments before building preset to catch malformed port specifications
        if let Err(e) = args.validate() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        let desc = args
            .preset_desc
            .clone()
            .unwrap_or_else(|| "User-defined preset".to_string());
        let preset = args.build_preset_from_args(name.clone(), desc);
        let mut mgr = port_kill::preset_manager::PresetManager::new();
        if let Err(e) = mgr.load_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        mgr.add_preset(preset);
        if let Err(e) = mgr.save_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        println!("✅ Saved preset '{}'.", name);
        return Ok(());
    }

    // Delete preset
    if let Some(name) = args.delete_preset.clone() {
        let mut mgr = port_kill::preset_manager::PresetManager::new();
        if let Err(e) = mgr.load_presets() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        match mgr.remove_preset(&name) {
            Some(_) => {
                if let Err(e) = mgr.save_presets() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                println!("🗑️  Deleted preset '{}'.", name);
            }
            None => {
                eprintln!("Preset '{}' not found.", name);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Apply preset if specified
    if let Some(preset_name) = args.preset.clone() {
        if let Err(e) = args.load_preset(&preset_name) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Set up logging level based on log_level argument
    let log_level = if args.verbose {
        // Verbose flag overrides log_level for backward compatibility
        "debug"
    } else {
        args.log_level.to_rust_log()
    };
    std::env::set_var("RUST_LOG", log_level);

    // Initialize logging
    env_logger::init();

    info!("Starting Port Kill application on Linux...");
    info!("Monitoring: {}", args.get_port_description());

    // Handle cache subcommand: route to console-like behavior
    if let Some(cache_cmd) = args.cache.clone() {
        let c = cache_cmd.args();
        if c.list || c.dry_run {
            let resp = list_caches(
                &c.lang,
                c.npx,
                c.js_pm,
                c.hf,
                c.torch,
                c.vercel,
                c.cloudflare,
                c.stale_days,
            )
            .await;
            if c.json {
                print_or_json(&resp, true);
            } else {
                print_list_table(&resp);
            }
            return Ok(());
        }
        if c.clean {
            let resp = clean_caches(
                &c.lang,
                c.npx,
                c.js_pm,
                c.safe_delete,
                c.force,
                c.hf,
                c.torch,
                c.vercel,
                c.cloudflare,
                c.stale_days,
            )
            .await;
            print_or_json(&resp, c.json);
            return Ok(());
        }
        if c.restore_last {
            let resp = restore_last_backup().await;
            print_or_json(&resp, c.json);
            if resp.error.is_some() {
                std::process::exit(1);
            }
            return Ok(());
        }
        if c.doctor {
            let report = doctor().await;
            print_or_json(&report, c.json);
            return Ok(());
        }
    }

    // Handle new lifecycle management features
    
    if let Some(port) = args.restart {
        let app = ConsolePortKillApp::new(args)?;
        app.restart_port(port).await?;
        return Ok(());
    }

    if args.show_restart_history {
        let app = ConsolePortKillApp::new(args)?;
        app.show_restart_history().await?;
        return Ok(());
    }

    if let Some(port) = args.clear_restart {
        let app = ConsolePortKillApp::new(args)?;
        app.clear_restart_history(port).await?;
        return Ok(());
    }

    if args.detect {
        let app = ConsolePortKillApp::new(args)?;
        app.detect_services().await?;
        return Ok(());
    }

    if let Some(service_name) = args.start.clone() {
        let app = ConsolePortKillApp::new(args)?;
        app.start_service(&service_name).await?;
        return Ok(());
    }

    if args.init_config {
        let app = ConsolePortKillApp::new(args)?;
        app.init_config().await?;
        return Ok(());
    }

    if args.up {
        let app = ConsolePortKillApp::new(args)?;
        app.orchestrate_up().await?;
        return Ok(());
    }

    if args.down {
        let app = ConsolePortKillApp::new(args)?;
        app.orchestrate_down().await?;
        return Ok(());
    }

    if let Some(service_name) = args.restart_service.clone() {
        let app = ConsolePortKillApp::new(args)?;
        app.orchestrate_restart(&service_name).await?;
        return Ok(());
    }

    if args.status {
        let app = ConsolePortKillApp::new(args)?;
        app.orchestrate_status().await?;
        return Ok(());
    }

    // Create and run the console application
    let app = ConsolePortKillApp::new(args)?;
    app.run().await?;

    info!("Port Kill application stopped");
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn main() {
    eprintln!("Error: This binary is only available on macOS, Windows, and Linux.");
    eprintln!("For other platforms, use the platform-specific binaries:");
    eprintln!("  - Console mode (all platforms): ./run.sh --console");
    std::process::exit(1);
}
