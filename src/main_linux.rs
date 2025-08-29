// Linux-specific main entry point
// This provides Linux tray support while maintaining all core functionality

use port_kill::{
    cli::Args,
    console_app::ConsolePortKillApp,
    types::{ProcessInfo, StatusBarInfo},
};
use tray_item::TrayItem;
use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

// GTK initialization for Linux
#[cfg(target_os = "linux")]
use gtk::prelude::*;
#[cfg(target_os = "linux")]
use gtk;



#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    
    info!("Starting Port Kill application on Linux...");
    info!("Monitoring: {}", args.get_port_description());
    
    // Check if running in console mode
    if args.console {
        // Use console mode (works identically to macOS)
        let console_app = ConsolePortKillApp::new(args)?;
        console_app.run().await
    } else {
        // Initialize GTK for Linux tray mode
        gtk::init().map_err(|e| anyhow::anyhow!("Failed to initialize GTK: {}", e))?;
        
        // Use Linux tray mode
        run_linux_tray_mode(args).await
    }
}

async fn run_linux_tray_mode(args: Args) -> Result<()> {
    info!("Starting Linux tray mode...");
    
    // Check if we're in a proper desktop environment
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let wayland_display = std::env::var("WAYLAND_DISPLAY").is_ok();
    
    info!("Display: {}, Wayland: {}", display, wayland_display);
    
    // Run diagnostics
    run_linux_diagnostics();
    
    // Set required environment variables for GTK
    if std::env::var("DISPLAY").is_err() {
        std::env::set_var("DISPLAY", ":0");
    }
    std::env::set_var("GTK_THEME", "Adwaita");
    
    // Try different GDK backends
    let backends = if wayland_display {
        vec!["wayland", "x11"]
    } else {
        vec!["x11", "wayland"]
    };
    
    let mut tray = None;
    let mut last_error = None;
    
    // Try to create tray with different backends
    for backend in backends {
        std::env::set_var("GDK_BACKEND", backend);
        info!("Trying GDK backend: {}", backend);
        
        match TrayItem::new("Port Kill", "Port Kill") {
            Ok(t) => {
                tray = Some(t);
                info!("Successfully created tray with backend: {}", backend);
                break;
            }
            Err(e) => {
                let error_msg = e.to_string();
                last_error = Some(e);
                error!("Failed to create tray with backend {}: {}", backend, error_msg);
                continue;
            }
        }
    }
    
    // If all backends failed, fall back to console mode
    let mut tray = match tray {
        Some(t) => t,
        None => {
            let error_msg = last_error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string());
            error!("Failed to create Linux tray item: {}", error_msg);
            error!("This might be due to:");
            error!("  - Missing GTK packages");
            error!("  - Running in a headless environment");
            error!("  - Display server issues");
            error!("  - Wayland/X11 compatibility issues");
            error!("Falling back to console mode...");
            println!("⚠️  System tray not available, switching to console mode");
            println!("💡 Common solutions:");
            println!("   1. Install GTK packages:");
            println!("      Ubuntu/Debian: sudo apt-get install libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev libxdo-dev");
            println!("      Fedora/RHEL: sudo dnf install atk-devel gdk-pixbuf2-devel gtk3-devel libxdo-devel");
            println!("      Arch Linux: sudo pacman -S atk gdk-pixbuf2 gtk3 libxdo");
            println!("   2. Check display environment:");
            println!("      echo $DISPLAY (should show :0 or similar)");
            println!("      echo $WAYLAND_DISPLAY (should be empty for X11)");
            println!("   3. Try running in a terminal emulator (not pure console)");
            println!("   4. Use console mode: ./run-linux.sh --console");
            println!("");
            
            // Fall back to console mode
            let console_args = Args {
                console: true,
                ..args
            };
            let console_app = ConsolePortKillApp::new(console_args)?;
            return console_app.run().await;
        }
    };
    
    info!("Linux tray created successfully!");
    println!("🔍 Look for the Port Kill icon in your system tray!");
    println!("   It should appear in your desktop environment's notification area.");
    
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
            
            // Get process information
            let (process_count, processes) = get_processes_on_ports(&args.get_ports_to_monitor(), &args);
            let status_info = StatusBarInfo::from_process_count(process_count);
            
            // Update tray tooltip (tray-item doesn't support dynamic tooltip updates)
            // The tooltip is set when creating the tray item
            
            // Print status to console as well
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
            }
            
            last_process_count = process_count;
        }
        

        
        // Small delay to prevent busy waiting
        thread::sleep(Duration::from_millis(100));
    }
    
    info!("Port Kill application exiting...");
    Ok(())
}

// Core process management functions (copied from app.rs to avoid dependencies)

fn get_processes_on_ports(ports: &[u16], args: &Args) -> (usize, HashMap<u16, ProcessInfo>) {
    // Build port range string for lsof
    let port_range = if ports.len() <= 10 {
        // For small number of ports, list them individually
        ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
    } else {
        // For large ranges, use range format
        format!("{}-{}", ports.first().unwrap_or(&0), ports.last().unwrap_or(&0))
    };
    
    // Use lsof to get detailed process information
    let output = std::process::Command::new("lsof")
        .args(&["-i", &format!(":{}", port_range), "-sTCP:LISTEN", "-P", "-n"])
        .output();
        
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut processes = HashMap::new();
            
            // Get ignore sets for efficient lookup
            let ignore_ports = args.get_ignore_ports_set();
            let ignore_processes = args.get_ignore_processes_set();
            
            for line in stdout.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    if let (Ok(pid), Ok(port)) = (parts[1].parse::<i32>(), parts[8].split(':').last().unwrap_or("0").parse::<u16>()) {
                        let command = parts[0].to_string();
                        let name = parts[0].to_string();
                        
                        // Check if this process should be ignored
                        let should_ignore = ignore_ports.contains(&port) || ignore_processes.contains(&name);
                        
                        if !should_ignore {
                            processes.insert(port, ProcessInfo {
                                pid,
                                port,
                                command,
                                name,
                                container_id: None,
                                container_name: None,
                            });
                        } else {
                            info!("Ignoring process {} (PID {}) on port {} (ignored by user configuration)", name, pid, port);
                        }
                    }
                }
            }
            
            (processes.len(), processes)
        }
        Err(_) => (0, HashMap::new())
    }
}

fn kill_all_processes(ports: &[u16], args: &Args) -> Result<()> {
    // Build port range string for lsof
    let port_range = if ports.len() <= 10 {
        // For small number of ports, list them individually
        ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
    } else {
        // For large ranges, use range format
        format!("{}-{}", ports.first().unwrap_or(&0), ports.last().unwrap_or(&0))
    };
    
    info!("Killing all processes on ports {}...", port_range);
    
    // Get all PIDs on the monitored ports
    let output = match std::process::Command::new("lsof")
        .args(&["-i", &format!(":{}", port_range), "-sTCP:LISTEN", "-P", "-n"])
        .output() {
        Ok(output) => output,
        Err(e) => {
            error!("Failed to run lsof command: {}", e);
            return Err(anyhow::anyhow!("Failed to run lsof: {}", e));
        }
    };
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    // Get ignore sets for efficient lookup
    let ignore_ports = args.get_ignore_ports_set();
    let ignore_processes = args.get_ignore_processes_set();
    
    let mut pids_to_kill = Vec::new();
    
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 9 {
            if let (Ok(pid), Ok(port)) = (parts[1].parse::<i32>(), parts[8].split(':').last().unwrap_or("0").parse::<u16>()) {
                let name = parts[0].to_string();
                
                // Check if this process should be ignored
                let should_ignore = ignore_ports.contains(&port) || ignore_processes.contains(&name);
                
                if !should_ignore {
                    pids_to_kill.push(pid);
                } else {
                    info!("Ignoring process {} (PID {}) on port {} during kill operation (ignored by user configuration)", name, pid, port);
                }
            }
        }
    }
    
    if pids_to_kill.is_empty() {
        info!("No processes found to kill (all were ignored or none found)");
        return Ok(());
    }
    
    info!("Found {} processes to kill (after filtering ignored processes)", pids_to_kill.len());
    
    for pid in pids_to_kill {
        info!("Attempting to kill process PID: {}", pid);
        match kill_process(pid) {
            Ok(_) => info!("Successfully killed process PID: {}", pid),
            Err(e) => error!("Failed to kill process {}: {}", pid, e),
        }
    }
    
    info!("Finished killing all processes");
    Ok(())
}

fn kill_process(pid: i32) -> Result<()> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    
    info!("Killing process PID: {} with SIGTERM", pid);
    
    // First try SIGTERM (graceful termination)
    match kill(Pid::from_raw(pid), Signal::SIGTERM) {
        Ok(_) => info!("SIGTERM sent to PID: {}", pid),
        Err(e) => {
            // Don't fail immediately, just log the error and continue
            error!("Failed to send SIGTERM to PID {}: {} (process may already be terminated)", pid, e);
        }
    }
    
    // Wait a bit for graceful termination
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Check if process is still running
    let still_running = std::process::Command::new("ps")
        .args(&["-p", &pid.to_string()])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
        
    if still_running {
        // Process still running, send SIGKILL
        info!("Process {} still running, sending SIGKILL", pid);
        match kill(Pid::from_raw(pid), Signal::SIGKILL) {
            Ok(_) => info!("SIGKILL sent to PID: {}", pid),
            Err(e) => {
                // Log error but don't fail the entire operation
                error!("Failed to send SIGKILL to PID {}: {} (process may be protected)", pid, e);
            }
        }
    } else {
        info!("Process {} terminated gracefully", pid);
    }
    
    Ok(())
}

fn kill_single_process(pid: i32, args: &Args) -> Result<()> {
    info!("Killing single process PID: {}", pid);
    
    // Check if this process should be ignored
    let ignore_ports = args.get_ignore_ports_set();
    let ignore_processes = args.get_ignore_processes_set();
    
    // Get process info to check if it should be ignored
    let output = std::process::Command::new("ps")
        .args(&["-p", &pid.to_string(), "-o", "comm="])
        .output();
        
    if let Ok(output) = output {
        let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Check if process name should be ignored
        if ignore_processes.contains(&process_name) {
            info!("Ignoring process {} (PID {}) - process name is in ignore list", process_name, pid);
            return Ok(());
        }
    }
    
    // Get port info to check if it should be ignored
    let output = std::process::Command::new("lsof")
        .args(&["-p", &pid.to_string(), "-i", "-P", "-n"])
        .output();
        
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                if let Ok(port) = parts[8].split(':').last().unwrap_or("0").parse::<u16>() {
                    if ignore_ports.contains(&port) {
                        info!("Ignoring process on port {} (PID {}) - port is in ignore list", port, pid);
                        return Ok(());
                    }
                }
            }
        }
    }
    
    // Process is not ignored, proceed with killing
    kill_process(pid)
}

fn run_linux_diagnostics() {
    println!("🔍 Linux Environment Diagnostics:");
    println!("==================================");
    
    // Check DISPLAY
    match std::env::var("DISPLAY") {
        Ok(display) => println!("✅ DISPLAY: {}", display),
        Err(_) => println!("❌ DISPLAY: Not set"),
    }
    
    // Check WAYLAND_DISPLAY
    match std::env::var("WAYLAND_DISPLAY") {
        Ok(wayland) => println!("✅ WAYLAND_DISPLAY: {}", wayland),
        Err(_) => println!("❌ WAYLAND_DISPLAY: Not set"),
    }
    
    // Check XDG_SESSION_TYPE
    match std::env::var("XDG_SESSION_TYPE") {
        Ok(session) => println!("✅ XDG_SESSION_TYPE: {}", session),
        Err(_) => println!("❌ XDG_SESSION_TYPE: Not set"),
    }
    
    // Check if we're in a terminal
    match std::env::var("TERM") {
        Ok(term) => println!("✅ TERM: {}", term),
        Err(_) => println!("❌ TERM: Not set"),
    }
    
    // Check if we're in SSH
    if std::env::var("SSH_CLIENT").is_ok() || std::env::var("SSH_CONNECTION").is_ok() {
        println!("⚠️  SSH: Detected SSH session");
    } else {
        println!("✅ SSH: Not detected");
    }
    
    // Check for common desktop environments
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".to_string());
    println!("✅ Desktop Environment: {}", desktop);
    
    // Check for GTK packages
    println!("\n🔧 GTK Package Check:");
    let gtk_check = std::process::Command::new("pkg-config")
        .args(&["--exists", "gtk+-3.0"])
        .output();
    
    match gtk_check {
        Ok(output) if output.status.success() => {
            println!("✅ GTK+3.0: Available");
            
            // Get GTK version
            let version_check = std::process::Command::new("pkg-config")
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
