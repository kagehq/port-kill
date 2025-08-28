use crate::{
    process_monitor::ProcessMonitor,
    tray_menu::TrayMenu,
    types::{ProcessUpdate, StatusBarInfo},
    cli::Args,
};
use std::collections::HashMap;
use anyhow::Result;
use crossbeam_channel::{bounded, Receiver};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::{
    menu::MenuEvent,
    TrayIcon, TrayIconBuilder,
};
use winit::event_loop::EventLoop;


pub struct PortKillApp {
    tray_icon: Arc<StdMutex<Option<TrayIcon>>>,
    menu_event_receiver: Receiver<MenuEvent>,
    process_monitor: Arc<Mutex<ProcessMonitor>>,
    update_receiver: Receiver<ProcessUpdate>,
    tray_menu: TrayMenu,
    args: Args,
}

impl PortKillApp {
    pub fn new(args: Args) -> Result<Self> {
        // Create channels for communication
        let (update_sender, update_receiver) = bounded(100);
        let (menu_sender, menu_event_receiver) = bounded(100);

        // Create process monitor with configurable ports
        let process_monitor = Arc::new(Mutex::new(ProcessMonitor::new(update_sender, args.get_ports_to_monitor(), args.docker)?));

        // Create tray menu
        let tray_menu = TrayMenu::new(menu_sender)?;

        Ok(Self {
            tray_icon: Arc::new(StdMutex::new(None)),
            menu_event_receiver,
            process_monitor,
            update_receiver,
            tray_menu,
            args,
        })
    }

    pub fn run(mut self) -> Result<()> {
        info!("Starting Port Kill application...");

        // Create event loop first (before any NSApplication initialization)
        let event_loop = EventLoop::new()?;
        
        // Now create the tray icon after the event loop is created
        info!("Creating tray icon...");
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Port Kill - Development Port Monitor")
            .with_menu(Box::new(self.tray_menu.menu.clone()))
            .with_icon(self.tray_menu.icon.clone())
            .build()?;
            
        info!("Tray icon created successfully!");
        
        // Store the tray icon
        if let Ok(mut tray_icon_guard) = self.tray_icon.lock() {
            *tray_icon_guard = Some(tray_icon);
        }
        
        // For now, let's manually check for processes every 5 seconds in the event loop
        let tray_icon = self.tray_icon.clone();
        let mut last_check = std::time::Instant::now();
        let mut last_process_count = 0;
        let mut last_menu_update = std::time::Instant::now();
        let is_killing_processes = Arc::new(AtomicBool::new(false));
        let mut confirmation_pending = false;

        // Give the tray icon time to appear
        info!("Waiting for tray icon to appear...");
        println!("🔍 Look for a white square with red/green center in your status bar!");
        println!("   It should be in the top-right area of your screen.");

        // Set up menu event handling
        let menu_event_receiver = self.menu_event_receiver.clone();
        
        // Run the event loop
        event_loop.run(move |_event, _elwt| {
            // Handle menu events (simplified to avoid crashes)
            if let Ok(event) = menu_event_receiver.try_recv() {
                info!("Menu event received: {:?}", event);
                info!("Menu event ID: {:?}", event.id);
                
                // For now, treat any menu click as "Kill All" to test the confirmation dialog
                // In a real implementation, we'd need to properly identify which menu item was clicked
                if !confirmation_pending {
                    // Show confirmation dialog
                    confirmation_pending = true;
                    println!("⚠️  Are you sure you want to kill all processes? Click 'Kill All Processes' again to confirm.");
                    
                    // Update menu to show confirmation state
                    if let Ok(tray_icon_guard) = tray_icon.lock() {
                        if let Some(ref icon) = *tray_icon_guard {
                            if let Ok(confirm_menu) = Self::create_confirmation_menu() {
                                icon.set_menu(Some(Box::new(confirm_menu)));
                            }
                        }
                    }
                } else {
                    // User confirmed, proceed with killing
                    confirmation_pending = false;
                    info!("User confirmed, starting process killing...");
                    is_killing_processes.store(true, Ordering::Relaxed);
                    
                    // Spawn a detached thread to kill processes
                    let ports_to_kill = self.args.get_ports_to_monitor();
                    let is_killing_clone = is_killing_processes.clone();
                    std::thread::spawn(move || {
                        // Add a longer delay to ensure the menu system is stable
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        info!("Starting process killing...");
                        match PortKillApp::kill_all_processes(&ports_to_kill) {
                            Ok(_) => {
                                info!("Process killing completed successfully");
                                // Reset the flag after a delay to allow menu updates again
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                is_killing_clone.store(false, Ordering::Relaxed);
                            }
                            Err(e) => {
                                error!("Failed to kill all processes: {}", e);
                                is_killing_clone.store(false, Ordering::Relaxed);
                            }
                        }
                    });
                }
            }
            
            // Check for processes every 5 seconds (less frequent to avoid crashes)
            if last_check.elapsed() >= std::time::Duration::from_secs(5) {
                last_check = std::time::Instant::now();
                
                // Get detailed process information
                let (process_count, processes) = Self::get_processes_on_ports(&self.args.get_ports_to_monitor());
                let status_info = StatusBarInfo::from_process_count(process_count);
                println!("🔄 Port Status: {} - {}", status_info.text, status_info.tooltip);
                
                // Print detected processes
                if process_count > 0 {
                    println!("📋 Detected Processes:");
                    for (port, process_info) in &processes {
                        if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                            println!("   • Port {}: {} [Docker: {}]", port, process_info.name, container_name);
                        } else if self.args.show_pid {
                            println!("   • Port {}: {} (PID {})", port, process_info.name, process_info.pid);
                        } else {
                            println!("   • Port {}: {}", port, process_info.name);
                        }
                    }
                }
                
                // Update tooltip, icon, and menu with better error handling
                if let Ok(tray_icon_guard) = tray_icon.lock() {
                    if let Some(ref icon) = *tray_icon_guard {
                        // Update tooltip
                        if let Err(e) = icon.set_tooltip(Some(&status_info.tooltip)) {
                            error!("Failed to update tooltip: {}", e);
                        }
                        
                        // Update icon with new status
                        if let Ok(new_icon) = TrayMenu::create_icon(&status_info.text) {
                            if let Err(e) = icon.set_icon(Some(new_icon)) {
                                error!("Failed to update icon: {}", e);
                            }
                        }
                        
                        // Update menu with current processes (with cooldown to prevent crashes)
                        if !is_killing_processes.load(Ordering::Relaxed) && 
                           process_count != last_process_count && 
                           last_menu_update.elapsed() >= std::time::Duration::from_secs(3) {
                            
                            // Only update menu if we have processes to show
                            if process_count > 0 {
                                match TrayMenu::create_menu(&processes, self.args.show_pid) {
                                    Ok(new_menu) => {
                                        icon.set_menu(Some(Box::new(new_menu)));
                                    }
                                    Err(e) => {
                                        error!("Failed to create menu: {}", e);
                                    }
                                }
                            } else {
                                // Create empty menu when no processes
                                match TrayMenu::create_menu(&HashMap::new(), self.args.show_pid) {
                                    Ok(empty_menu) => {
                                        icon.set_menu(Some(Box::new(empty_menu)));
                                    }
                                    Err(e) => {
                                        error!("Failed to create empty menu: {}", e);
                                    }
                                }
                            }
                            last_process_count = process_count;
                            last_menu_update = std::time::Instant::now();
                        }
                    }
                }
            }
        })?;

        Ok(())
    }

    fn create_confirmation_menu() -> Result<tray_icon::menu::Menu> {
        let menu = tray_icon::menu::Menu::new();

        // Add confirmation message (make it clickable)
        let confirm_item = tray_icon::menu::MenuItem::new("⚠️  Click again to confirm Kill All", true, None);
        menu.append(&confirm_item)?;

        // Add separator
        let separator = tray_icon::menu::PredefinedMenuItem::separator();
        menu.append(&separator)?;

        // Add "Cancel" option
        let cancel_item = tray_icon::menu::MenuItem::new("Cancel", true, None);
        menu.append(&cancel_item)?;

        Ok(menu)
    }

    fn get_processes_on_ports(ports: &[u16]) -> (usize, HashMap<u16, crate::types::ProcessInfo>) {
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
                
                for line in stdout.lines().skip(1) { // Skip header
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 9 {
                        if let (Ok(pid), Ok(port)) = (parts[1].parse::<i32>(), parts[8].split(':').last().unwrap_or("0").parse::<u16>()) {
                            let command = parts[0].to_string();
                            let name = parts[0].to_string();
                            
                            processes.insert(port, crate::types::ProcessInfo {
                                pid,
                                port,
                                command,
                                name,
                                container_id: None,
                                container_name: None,
                            });
                        }
                    }
                }
                
                (processes.len(), processes)
            }
            Err(_) => (0, HashMap::new())
        }
    }

    fn kill_all_processes(ports: &[u16]) -> Result<()> {
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
            .args(&["-ti", &format!(":{}", port_range), "-sTCP:LISTEN"])
            .output() {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to run lsof command: {}", e);
                return Err(anyhow::anyhow!("Failed to run lsof: {}", e));
            }
        };
            
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pids: Vec<&str> = stdout.lines().filter(|line| !line.trim().is_empty()).collect();
        
        if pids.is_empty() {
            info!("No processes found to kill");
            return Ok(());
        }
        
        info!("Found {} processes to kill", pids.len());
        
        for pid_str in pids {
            if let Ok(pid) = pid_str.parse::<i32>() {
                info!("Attempting to kill process PID: {}", pid);
                match Self::kill_process(pid) {
                    Ok(_) => info!("Successfully killed process PID: {}", pid),
                    Err(e) => error!("Failed to kill process {}: {}", pid, e),
                }
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
                warn!("Failed to send SIGTERM to PID {}: {} (process may already be terminated)", pid, e);
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
                    warn!("Failed to send SIGKILL to PID {}: {} (process may be protected)", pid, e);
                }
            }
        } else {
            info!("Process {} terminated gracefully", pid);
        }
        
        Ok(())
    }
}
