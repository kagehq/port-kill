#[cfg(target_os = "macos")]
use anyhow::Result;
#[cfg(target_os = "macos")]
use log::info;
#[cfg(target_os = "macos")]
use port_kill::{app::PortKillApp, cli::Args};
#[cfg(target_os = "macos")]
use port_kill::cache::{list::{list_caches, print_list_table}, clean::clean_caches, restore::restore_last_backup, doctor::doctor};
#[cfg(target_os = "macos")]
use port_kill::cache::output::print_or_json;
use port_kill::update_check;
#[cfg(target_os = "macos")]
use clap::Parser;

#[cfg(target_os = "macos")]
fn main() -> Result<()> {
    // Parse command-line arguments
    let mut args = Args::parse();
    
    // Handle update check
    if args.check_updates {
        let current_version = env!("CARGO_PKG_VERSION");
        match update_check::check_for_updates(current_version) {
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
    
    // Check for updates in background (non-blocking)
    let current_version = env!("CARGO_PKG_VERSION");
    if let Ok(Some(update_info)) = update_check::check_for_updates(current_version) {
        update_check::print_update_notification(&update_info);
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
        let desc = args.preset_desc.clone().unwrap_or_else(|| "User-defined preset".to_string());
        let preset = args.build_preset_from_args(name.clone(), desc);
        let mut mgr = port_kill::preset_manager::PresetManager::new();
        if let Err(e) = mgr.load_presets() { eprintln!("Error: {}", e); std::process::exit(1); }
        mgr.add_preset(preset);
        if let Err(e) = mgr.save_presets() { eprintln!("Error: {}", e); std::process::exit(1); }
        println!("✅ Saved preset '{}'.", name);
        return Ok(());
    }

    // Delete preset
    if let Some(name) = args.delete_preset.clone() {
        let mut mgr = port_kill::preset_manager::PresetManager::new();
        if let Err(e) = mgr.load_presets() { eprintln!("Error: {}", e); std::process::exit(1); }
        match mgr.remove_preset(&name) {
            Some(_) => {
                if let Err(e) = mgr.save_presets() { eprintln!("Error: {}", e); std::process::exit(1); }
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
            let resp = tokio::runtime::Runtime::new().unwrap().block_on(list_caches(&c.lang, c.npx, c.js_pm, c.hf, c.torch, c.vercel, c.cloudflare, c.stale_days));
            if c.json {
                print_or_json(&resp, true);
            } else {
                print_list_table(&resp);
            }
            return Ok(());
        }
        if c.clean {
            let resp = tokio::runtime::Runtime::new().unwrap().block_on(clean_caches(&c.lang, c.npx, c.js_pm, c.safe_delete, c.force, c.hf, c.torch, c.vercel, c.cloudflare, c.stale_days));
            print_or_json(&resp, c.json);
            return Ok(());
        }
        if c.restore_last {
            let resp = tokio::runtime::Runtime::new().unwrap().block_on(restore_last_backup());
            print_or_json(&resp, c.json);
            return Ok(());
        }
        if c.doctor {
            let report = tokio::runtime::Runtime::new().unwrap().block_on(doctor());
            print_or_json(&report, c.json);
            return Ok(());
        }
    }

    // Create and run the application
    let app = PortKillApp::new(args)?;
    app.run()?;

    info!("Port Kill application stopped");
    Ok(())
}

#[cfg(target_os = "windows")]
use anyhow::Result;
#[cfg(target_os = "windows")]
use log::info;
#[cfg(target_os = "windows")]
use port_kill::{console_app::ConsolePortKillApp, cli::Args};
#[cfg(target_os = "windows")]
use clap::Parser;

#[cfg(target_os = "windows")]
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let mut args = Args::parse();
    
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

    // Create and run the console application
    let app = ConsolePortKillApp::new(args)?;
    app.run().await?;

    info!("Port Kill application stopped");
    Ok(())
}

#[cfg(target_os = "linux")]
use anyhow::Result;
#[cfg(target_os = "linux")]
use log::info;
#[cfg(target_os = "linux")]
use port_kill::{console_app::ConsolePortKillApp, cli::Args};
#[cfg(target_os = "linux")]
use clap::Parser;

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let mut args = Args::parse();
    
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
