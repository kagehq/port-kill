use port_kill::{
    cli::Args,
    console_app::ConsolePortKillApp,
    types::{ProcessInfo, StatusBarInfo},
    process_monitor::{get_processes_on_ports, kill_all_processes, kill_single_process},
};
use tray_item::TrayItem;
use anyhow::{Result, Context};
use clap::Parser;
use log::{error, info};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
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
    
    // Check if running in console mode
    if args.console {
        // Use console mode (works identically to macOS/Linux)
        let console_app = ConsolePortKillApp::new(args)?;
        console_app.run().await
    } else {
        // Use Windows tray mode; on failure, fall back to console
        match run_windows_tray_mode(args.clone()) {
            Ok(()) => Ok(()),
            Err(e) => {
                log::warn!("Tray mode failed on Windows ({}). Falling back to console mode...", e);
                let console_app = ConsolePortKillApp::new(args)?;
                console_app.run().await
            }
        }
    }
}

fn run_windows_tray_mode(args: Args) -> Result<()> {
    info!("Starting Windows tray mode...");
    
    // Create the tray item with embedded icon and multiple resource fallbacks
    // When build.rs embeds assets/port-kill.ico, the default resource name is IDI_APPLICATION.
    // Try a few common names to maximize compatibility.
    let mut tray = TrayItem::new("Port Kill", tray_item::IconSource::Resource("APPICON"))
        .or_else(|_| TrayItem::new("Port Kill", tray_item::IconSource::Resource("IDI_APPLICATION")))
        .or_else(|_| TrayItem::new("Port Kill", tray_item::IconSource::Resource("PORT_KILL")))
        .or_else(|_| TrayItem::new("Port Kill", tray_item::IconSource::Resource("IDI_APPLICATION")))
        .or_else(|_| TrayItem::new("Port Kill", tray_item::IconSource::Resource("APPLICATION")))
        .or_else(|_| TrayItem::new("Port Kill", tray_item::IconSource::Resource("MAINICON")))
        .map_err(|e| {
            anyhow::anyhow!("Failed to create Windows tray item after fallbacks: {}", e)
        })?;
    
    info!("Windows tray created successfully!");
    println!("🔍 Look for the Port Kill icon in your system tray!");
    println!("   It should appear in your Windows notification area.");
    
    // Create channels for communication
    let (menu_sender, menu_receiver) = std::sync::mpsc::channel();
    
    // Add menu items
    let sender_clone = menu_sender.clone();
    tray.add_menu_item("Kill All Processes", move || {
        if let Err(e) = sender_clone.send("kill_all") {
            error!("Failed to send kill_all event: {}", e);
        }
    }).map_err(|e| anyhow::anyhow!("Failed to add Kill All menu item: {}", e))?;
    
    let sender_clone = menu_sender.clone();
    tray.add_menu_item("Quit", move || {
        if let Err(e) = sender_clone.send("quit") {
            error!("Failed to send quit event: {}", e);
        }
    }).map_err(|e| anyhow::anyhow!("Failed to add Quit menu item: {}", e))?;
    
    // Main monitoring loop
    let mut last_check = std::time::Instant::now();
    let mut last_process_count = 0;
    let mut last_processes = HashMap::new();
    
    loop {
        // Check for menu events
        if let Ok(event) = menu_receiver.try_recv() {
            match event {
                "kill_all" => {
                    info!("Kill All Processes clicked");
                    let ports_to_kill = args.get_ports_to_monitor();
                    if let Err(e) = kill_all_processes(&ports_to_kill, &args) {
                        error!("Failed to kill all processes: {}", e);
                    } else {
                        println!("✅ All processes killed successfully");
                    }
                }
                "quit" => {
                    info!("Quit clicked, exiting...");
                    break;
                }
                _ => {
                    info!("Unknown menu event: {}", event);
                }
            }
        }
        
        // Check for processes every 5 seconds
        if last_check.elapsed() >= Duration::from_secs(5) {
            last_check = std::time::Instant::now();
            
            // Get process information with error handling
            let (process_count, processes) = match std::panic::catch_unwind(|| {
                get_processes_on_ports(&args.get_ports_to_monitor(), &args)
            }) {
                Ok(result) => result,
                Err(e) => {
                    error!("Panic caught while getting processes: {:?}", e);
                    (0, HashMap::new())
                }
            };
            let status_info = StatusBarInfo::from_process_count(process_count);
            
            // Only update if processes have actually changed
            if process_count != last_process_count || processes != last_processes {
                info!("Process list changed: {} processes (was: {})", process_count, last_process_count);
                
                // Print status to console
                println!("🔄 Port Status: {} - {}", status_info.text, status_info.tooltip);
                
                // Print detected processes
                if process_count > 0 {
                    println!("📋 Detected Processes:");
                    for (port, process_info) in &processes {
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
                
                // Update our tracking
                last_process_count = process_count;
                last_processes = processes;
            }
        }
        
        // Small delay to prevent busy waiting
        thread::sleep(Duration::from_millis(100));
    }
    
    info!("Port Kill application exiting...");
    Ok(())
}








