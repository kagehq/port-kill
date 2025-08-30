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
        // Use Windows tray mode
        run_windows_tray_mode(args)
    }
}

fn run_windows_tray_mode(args: Args) -> Result<()> {
    info!("Starting Windows tray mode...");
    
    // Create the tray item
    let mut tray = TrayItem::new("Port Kill", "Port Kill").map_err(|e| {
        anyhow::anyhow!("Failed to create Windows tray item: {}", e)
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

// Windows-specific process management functions
fn get_processes_on_ports(ports: &[u16], args: &Args) -> (usize, HashMap<u16, ProcessInfo>) {
    // Build port range string for netstat
    let port_range = if ports.len() <= 10 {
        // For small number of ports, list them individually
        ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
    } else {
        // For large ranges, use range format
        format!("{}-{}", ports.first().unwrap_or(&0), ports.last().unwrap_or(&0))
    };
    
    info!("Scanning for processes on ports: {}", port_range);
    
    // Use netstat to get process information on Windows
    let output = match std::process::Command::new("netstat")
        .args(&["-ano"])
        .output() {
        Ok(output) => output,
        Err(e) => {
            error!("Failed to run netstat command: {}", e);
            return (0, HashMap::new());
        }
    };
        
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut processes = HashMap::new();
            
            // Get ignore sets for efficient lookup
            let ignore_ports = args.get_ignore_ports_set();
            let ignore_processes = args.get_ignore_processes_set();
            
            for line in stdout.lines() {
                // Parse netstat output format: Proto Local Address Foreign Address State PID
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    // Extract port from local address (e.g., "0.0.0.0:3000")
                    if let Some(port_str) = parts[1].split(':').last() {
                        if let Ok(port) = port_str.parse::<u16>() {
                            // Check if this port is in our monitoring range
                            if ports.contains(&port) {
                                if let Ok(pid) = parts[4].parse::<i32>() {
                                    // Get process name using tasklist
                                    let process_name = get_process_name_by_pid(pid).unwrap_or_else(|| "unknown".to_string());
                                    
                                    // Check if this process should be ignored
                                    let should_ignore = ignore_ports.contains(&port) || ignore_processes.contains(&process_name);
                                    
                                    if !should_ignore {
                                        processes.insert(port, ProcessInfo {
                                            pid,
                                            port,
                                            command: process_name.clone(),
                                            name: process_name,
                                            container_id: None,
                                            container_name: None,
                                        });
                                    } else {
                                        info!("Ignoring process {} (PID {}) on port {} (ignored by user configuration)", process_name, pid, port);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            (processes.len(), processes)
        }
        Err(_) => (0, HashMap::new())
    }
}

fn get_process_name_by_pid(pid: i32) -> Option<String> {
    let output = std::process::Command::new("tasklist")
        .args(&["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Parse CSV format: "process.exe","PID","Session Name","Session#","Mem Usage"
                if let Some(name_part) = line.split(',').next() {
                    // Remove quotes and .exe extension
                    let name = name_part.trim_matches('"');
                    if let Some(name_without_ext) = name.strip_suffix(".exe") {
                        return Some(name_without_ext.to_string());
                    }
                    return Some(name.to_string());
                }
            }
        }
        Err(e) => {
            error!("Failed to get process name for PID {}: {}", pid, e);
        }
    }
    
    None
}

fn kill_all_processes(ports: &[u16], args: &Args) -> Result<()> {
    // Build port range string for netstat
    let port_range = if ports.len() <= 10 {
        // For small number of ports, list them individually
        ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
    } else {
        // For large ranges, use range format
        format!("{}-{}", ports.first().unwrap_or(&0), ports.last().unwrap_or(&0))
    };
    
    info!("Killing all processes on ports {}...", port_range);
    
    // Get all PIDs on the monitored ports using netstat
    let output = match std::process::Command::new("netstat")
        .args(&["-ano"])
        .output() {
        Ok(output) => output,
        Err(e) => {
            error!("Failed to run netstat command: {}", e);
            return Err(anyhow::anyhow!("Failed to run netstat: {}", e));
        }
    };
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    // Get ignore sets for efficient lookup
    let ignore_ports = args.get_ignore_ports_set();
    let ignore_processes = args.get_ignore_processes_set();
    
    let mut pids_to_kill = Vec::new();
    
    for line in lines {
        // Parse netstat output format: Proto Local Address Foreign Address State PID
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            // Extract port from local address (e.g., "0.0.0.0:3000")
            if let Some(port_str) = parts[1].split(':').last() {
                if let Ok(port) = port_str.parse::<u16>() {
                    // Check if this port is in our monitoring range
                    if ports.contains(&port) {
                        if let Ok(pid) = parts[4].parse::<i32>() {
                            // Get process name
                            let process_name = get_process_name_by_pid(pid).unwrap_or_else(|| "unknown".to_string());
                            
                            // Check if this process should be ignored
                            let should_ignore = ignore_ports.contains(&port) || ignore_processes.contains(&process_name);
                            
                            if !should_ignore {
                                pids_to_kill.push(pid);
                            } else {
                                info!("Ignoring process {} (PID {}) on port {} during kill operation (ignored by user configuration)", process_name, pid, port);
                            }
                        }
                    }
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
        info!("Killing process PID: {}", pid);
        
        // Use taskkill to terminate the process
        let kill_output = std::process::Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .output();
        
        match kill_output {
            Ok(output) => {
                if output.status.success() {
                    info!("Successfully killed process PID: {}", pid);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to kill process PID {}: {}", pid, stderr);
                }
            }
            Err(e) => {
                error!("Failed to execute taskkill for PID {}: {}", pid, e);
            }
        }
    }
    
    Ok(())
}

fn kill_single_process(pid: i32, args: &Args) -> Result<()> {
    info!("Killing single process PID: {}", pid);
    
    // Use taskkill to terminate the process
    let output = std::process::Command::new("taskkill")
        .args(&["/PID", &pid.to_string(), "/F"])
        .output()
        .context("Failed to execute taskkill command")?;
    
    if output.status.success() {
        info!("Successfully killed process PID: {}", pid);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Failed to kill process PID {}: {}", pid, stderr))
    }
}
