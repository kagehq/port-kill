// Linux-specific main entry point
// This provides Linux tray support while maintaining all core functionality

use port_kill::{
    cli::Args,
    console_app::ConsolePortKillApp,
    types::StatusBarInfo,
    process_monitor::{get_processes_on_ports, kill_all_processes},
};
use tray_item::TrayItem;
use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::process;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    // Set up logging level based on log_level argument
    let log_level = if args.verbose {
        // Verbose flag overrides log_level for backward compatibility
        "debug"
    } else {
        args.log_level.to_rust_log()
    };
    env::set_var("RUST_LOG", log_level);

    // Initialize logging
    env_logger::init();
    
    info!("Starting Port Kill application on Linux...");
    info!("Monitoring: {}", args.get_port_description());

    // Check if console mode is requested
    if args.console {
        info!("Starting console mode...");
        let console_app = ConsolePortKillApp::new(args)?;
        console_app.run().await?;
        return Ok(());
    }

    // Try to start tray mode, fallback to console if it fails
    match start_tray_mode(args.clone()).await {
        Ok(_) => {
            info!("Tray mode completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Tray mode failed: {}", e);
            println!("⚠️  Tray mode failed, falling back to console mode...");
            println!("   Error: {}", e);
            println!("   Running diagnostics...");
            run_linux_diagnostics();
            
            info!("Starting console mode as fallback...");
            let console_args = args.clone();
            let console_app = ConsolePortKillApp::new(console_args)?;
            console_app.run().await?;
            Ok(())
        }
    }
}

async fn start_tray_mode(args: Args) -> Result<()> {
    info!("Starting Linux tray mode...");
    
    // Create tray icon
    let mut tray = TrayItem::new("Port Kill", "Port Kill").map_err(|e| {
        anyhow::anyhow!("Failed to create tray item: {}", e)
    })?;
    
    let mut last_process_count = 0;
    let mut last_processes = HashMap::new();
    
    // Set up tray icon click handler
    tray.add_menu_item("Kill All Processes", move || {
        info!("Kill All Processes clicked");
        let ports_to_kill = args.get_ports_to_monitor();
        if let Err(e) = kill_all_processes(&ports_to_kill, &args) {
            error!("Failed to kill all processes: {}", e);
        }
    })?;
    
    tray.add_menu_item("Quit", move || {
        info!("Quit clicked, exiting gracefully...");
        process::exit(0);
    })?;
    
    info!("Tray icon created successfully!");
    println!("🔍 Look for the Port Kill icon in your system tray!");
    println!("💡 When in full-screen mode, use console mode: ./run.sh --console --ports 3000,8000");
    
    // Main monitoring loop
    loop {
        thread::sleep(Duration::from_secs(5));
        
        // Get current processes
        let (process_count, processes) = 
            get_processes_on_ports(&args.get_ports_to_monitor(), &args);
        
        // Update status
        let status_info = StatusBarInfo::from_process_count(process_count);
        println!("🔄 Port Status: {} - {}", status_info.text, status_info.tooltip);
        
        // Update current processes
        last_process_count = process_count;
        last_processes = processes;
        
        // Print detected processes
        if process_count > 0 {
            println!("📋 Detected Processes:");
            for (port, process_info) in &last_processes {
                if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                    println!("   • Port {}: {} [Docker: {}]", port, process_info.name, container_name);
                } else if args.show_pid {
                    println!("   • Port {}: {} (PID {})", port, process_info.name, process_info.pid);
                } else {
                    println!("   • Port {}: {}", port, process_info.name);
                }
            }
        } else {
            println!("📋 No processes detected");
        }
        
        // Update menu with current processes
        if process_count != last_process_count {
            info!("Process count changed from {} to {}, updating menu...", last_process_count, process_count);
            
            // Note: tray-item doesn't support dynamic menu updates easily
            // For now, we'll just monitor and display in console
        }
    }
}

fn run_linux_diagnostics() {
    println!("🔍 Linux Environment Diagnostics:");
    println!("==================================");
    
    // Check DISPLAY
    match env::var("DISPLAY") {
        Ok(display) => println!("✅ DISPLAY: {}", display),
        Err(_) => println!("❌ DISPLAY: Not set"),
    }
    
    // Check WAYLAND_DISPLAY
    match env::var("WAYLAND_DISPLAY") {
        Ok(wayland) => println!("✅ WAYLAND_DISPLAY: {}", wayland),
        Err(_) => println!("❌ WAYLAND_DISPLAY: Not set"),
    }
    
    // Check XDG_SESSION_TYPE
    match env::var("XDG_SESSION_TYPE") {
        Ok(session) => println!("✅ XDG_SESSION_TYPE: {}", session),
        Err(_) => println!("❌ XDG_SESSION_TYPE: Not set"),
    }
    
    // Check if we're in a terminal
    match env::var("TERM") {
        Ok(term) => println!("✅ TERM: {}", term),
        Err(_) => println!("❌ TERM: Not set"),
    }
    
    // Check if we're in SSH
    if env::var("SSH_CLIENT").is_ok() || env::var("SSH_CONNECTION").is_ok() {
        println!("⚠️  SSH: Detected SSH session");
    } else {
        println!("✅ SSH: Not detected");
    }
    
    // Check for common desktop environments
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".to_string());
    println!("✅ Desktop Environment: {}", desktop);
    
    // Check for GTK packages
    println!("\n🔧 GTK Package Check:");
    let gtk_check = process::Command::new("pkg-config")
        .args(&["--exists", "gtk+-3.0"])
        .output();
    
    match gtk_check {
        Ok(output) if output.status.success() => {
            println!("✅ GTK+3.0: Available");
            
            // Get GTK version
            let version_check = process::Command::new("pkg-config")
                .args(&["--modversion", "gtk+-3.0"])
                .output();
            
            if let Ok(version_output) = version_check {
                let version_str = String::from_utf8_lossy(&version_output.stdout);
                let version = version_str.trim();
                println!("✅ GTK Version: {}", version);
            }
        }
        _ => println!("❌ GTK+3.0: Not available (install GTK development packages)"),
    }
    
    println!("");
}
