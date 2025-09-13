use crate::{
    tray_menu::TrayMenu,
    types::StatusBarInfo,
    cli::Args,
};
use std::collections::HashMap;
use anyhow::Result;
use crossbeam_channel::{bounded, Receiver};
use log::{error, info, warn};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "macos")]
use tray_icon::{
    menu::MenuEvent,
    TrayIcon, TrayIconBuilder,
};
#[cfg(target_os = "macos")]
use winit::event_loop::EventLoop;


#[cfg(target_os = "macos")]
pub struct PortKillApp {
    tray_icon: Arc<StdMutex<Option<TrayIcon>>>,
    menu_event_receiver: Receiver<MenuEvent>,
    tray_menu: TrayMenu,
    args: Args,
    current_processes: Arc<StdMutex<HashMap<u16, crate::types::ProcessInfo>>>,
}

#[cfg(target_os = "macos")]
impl PortKillApp {
    pub fn new(args: Args) -> Result<Self> {
        // Create channels for communication
        let (menu_sender, menu_event_receiver) = bounded(100);

        // Create tray menu
        let tray_menu = TrayMenu::new(menu_sender)?;

        Ok(Self {
            tray_icon: Arc::new(StdMutex::new(None)),
            menu_event_receiver,
            tray_menu,
            args,
            current_processes: Arc::new(StdMutex::new(HashMap::new())),
        })
    }

    pub fn run(self) -> Result<()> {
        info!("Starting Port Kill application...");

        // Create event loop first (before any NSApplication initialization)
        let event_loop = EventLoop::new()?;
        

        
        // Now create the tray icon after the event loop is created
        info!("Creating tray icon...");
        let initial_menu = self.tray_menu.get_current_menu()?;
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Port Kill - Development Port Monitor (Click or press Cmd+Shift+P)")
            .with_menu(Box::new(initial_menu))
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

        // Give the tray icon time to appear
        info!("Waiting for tray icon to appear...");
        println!("🔍 Look for a white square with red/green center in your status bar!");
        println!("   It should be in the top-right area of your screen.");
        println!("💡 When in full-screen mode, use console mode: ./run.sh --console --ports 3000,8000");

        // Set up menu event handling
        let menu_event_receiver = self.menu_event_receiver.clone();
        let current_processes = self.current_processes.clone();
        let args = self.args.clone();
        
        // Run the event loop
        event_loop.run(move |_event, _elwt| {
            // Handle menu events with crash-safe approach
            if let Ok(event) = menu_event_receiver.try_recv() {
                info!("Menu event received: {:?}", event);
                
                // Only process if we're not already killing processes
                if !is_killing_processes.load(Ordering::Relaxed) {
                    info!("Processing menu event, starting process killing...");
                    is_killing_processes.store(true, Ordering::Relaxed);
                    
                    // Get current processes for menu handling
                    let current_processes_clone = current_processes.clone();
                    let is_killing_clone = is_killing_processes.clone();
                    let args_clone = args.clone();
                    
                    std::thread::spawn(move || {
                        // Add a delay to ensure the menu system is stable
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        
                        // Handle different menu actions based on event
                        let result = if let Ok(current_processes_guard) = current_processes_clone.lock() {
                            let processes = &*current_processes_guard;
                            
                            // Parse the menu event to determine action
                            let menu_id_str = event.id.0.clone();
                            info!("Menu ID: {}", menu_id_str);
                            
                            // Menu structure:
                            // 0: "Kill All Processes"
                            // 1: separator
                            // 2+: individual processes
                            // last: separator + "Quit"
                            
                            if menu_id_str == "0" {
                                // "Kill All Processes" clicked
                                info!("Kill All Processes clicked, killing all processes...");
                                let ports_to_kill = args_clone.get_ports_to_monitor();
                                Self::kill_all_processes(&ports_to_kill, &args_clone)
                            } else if let Ok(menu_index) = menu_id_str.parse::<usize>() {
                                // Calculate the actual menu structure
                                let kill_all_index = 0;
                                let separator1_index = 1;
                                let first_process_index = 2;
                                let last_process_index = first_process_index + processes.len() - 1;
                                let separator2_index = last_process_index + 1;
                                let quit_index = separator2_index + 1;
                                
                                info!("Menu structure: KillAll={}, Sep1={}, Processes={}-{}, Sep2={}, Quit={}", 
                                      kill_all_index, separator1_index, first_process_index, last_process_index, 
                                      separator2_index, quit_index);
                                
                                if menu_index >= first_process_index && menu_index <= last_process_index {
                                    // Individual process clicked
                                    let process_index = menu_index - first_process_index;
                                    let process_entries: Vec<_> = processes.iter().collect();
                                    if process_index < process_entries.len() {
                                        let (port, process_info) = process_entries[process_index];
                                        info!("Killing individual process on port {} with PID {}", port, process_info.pid);
                                        Self::kill_single_process(process_info.pid, &args_clone)
                                    } else {
                                        error!("Invalid process index: {} (max: {})", process_index, process_entries.len());
                                        Ok(())
                                    }
                                } else if menu_index == quit_index {
                                    // "Quit" clicked - just exit gracefully without killing processes
                                    info!("Quit clicked, exiting gracefully...");
                                    // Exit the application gracefully without killing processes
                                    std::process::exit(0);
                                } else {
                                    // Invalid menu index
                                    error!("Invalid menu index: {} (processes: {}, valid range: {}-{})", 
                                           menu_index, processes.len(), first_process_index, quit_index);
                                    Ok(())
                                }
                            } else {
                                // Default to kill all processes for unknown menu items
                                info!("Unknown menu item clicked: {}, killing all processes...", menu_id_str);
                                let ports_to_kill = args_clone.get_ports_to_monitor();
                                Self::kill_all_processes(&ports_to_kill, &args_clone)
                            }
                        } else {
                            error!("Failed to access current processes");
                            Ok(())
                        };
                        
                        match result {
                            Ok(_) => {
                                info!("Process killing completed successfully");
                                // Reset the flag after a delay to allow menu updates again
                                std::thread::sleep(std::time::Duration::from_secs(1));
                                is_killing_clone.store(false, Ordering::Relaxed);
                            }
                            Err(e) => {
                                error!("Failed to kill processes: {}", e);
                                is_killing_clone.store(false, Ordering::Relaxed);
                            }
                        }
                    });
                } else {
                    info!("Menu event received but already killing processes, ignoring");
                }
            }
            
            // Check for processes every 5 seconds (less frequent to avoid crashes)
            if last_check.elapsed() >= std::time::Duration::from_secs(5) {
                last_check = std::time::Instant::now();
                
                // Get detailed process information with crash-safe approach
                let (process_count, processes) = if args.verbose {
                    // Use ProcessMonitor for verbose information
                    match std::panic::catch_unwind(|| {
                        Self::get_processes_on_ports_verbose(&args.get_ports_to_monitor(), &args)
                    }) {
                        Ok(result) => result,
                        Err(e) => {
                            error!("Panic caught while getting verbose processes: {:?}", e);
                            (0, HashMap::new())
                        }
                    }
                } else {
                    // Use simple process detection for non-verbose mode
                    match std::panic::catch_unwind(|| {
                        Self::get_processes_on_ports(&args.get_ports_to_monitor(), &args)
                    }) {
                        Ok(result) => result,
                        Err(e) => {
                            error!("Panic caught while getting processes: {:?}", e);
                            (0, HashMap::new())
                        }
                    }
                };
                
                let status_info = StatusBarInfo::from_process_count(process_count);
                println!("🔄 Port Status: {} - {}", status_info.text, status_info.tooltip);
                
                // Update current processes
                if let Ok(mut current_processes_guard) = current_processes.lock() {
                    *current_processes_guard = processes.clone();
                }
                
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
                
                // Update tooltip and icon (avoid menu updates to prevent crashes)
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
                        
                        // Only update menu if process count changed significantly and we're not killing processes
                        // Add extra delay after killing processes to prevent crashes
                        let process_count_changed = process_count != last_process_count;
                        let enough_time_passed = last_menu_update.elapsed() >= std::time::Duration::from_secs(10); // Increased delay
                        let not_killing = !is_killing_processes.load(Ordering::Relaxed);
                        
                        if not_killing && process_count_changed && enough_time_passed {
                            info!("Process count changed from {} to {}, updating menu...", last_process_count, process_count);

                            // Additional validation: ensure all processes in the list are still running
                            let valid_processes: HashMap<u16, crate::types::ProcessInfo> = processes
                                .iter()
                                .filter(|(_, process_info)| Self::is_process_still_running(process_info.pid))
                                .map(|(port, process_info)| (*port, process_info.clone()))
                                .collect();
                            
                            let valid_process_count = valid_processes.len();
                            
                            if valid_process_count != process_count {
                                info!("Process count validation: {} processes reported, {} still running, updating with valid processes", 
                                      process_count, valid_process_count);
                            }
                            
                            // Only proceed if we have valid processes
                            if !valid_processes.is_empty() {
                                // Use a try-catch approach to prevent crashes
                                match std::panic::catch_unwind(|| {
                                    TrayMenu::create_menu_with_verbose(&valid_processes, args.show_pid, args.verbose)
                                }) {
                                    Ok(Ok(new_menu)) => {
                                        // Set the new menu on the tray icon
                                        icon.set_menu(Some(Box::new(new_menu)));
                                        last_process_count = valid_process_count;
                                        last_menu_update = std::time::Instant::now();
                                        info!("Menu updated successfully for {} processes", valid_process_count);
                                    }
                                    Ok(Err(e)) => {
                                        error!("Failed to create menu: {}", e);
                                    }
                                    Err(e) => {
                                        error!("Menu creation panicked: {:?}, skipping menu update", e);
                                    }
                                }
                            } else {
                                info!("No valid processes found, skipping menu update");
                                last_process_count = 0;
                                last_menu_update = std::time::Instant::now();
                            }
                        } else if process_count_changed {
                            info!("Process count changed from {} to {} but skipping menu update (killing: {}, time passed: {})",
                                  last_process_count, process_count, !not_killing, enough_time_passed);
                        }
                    }
                }
            }
        })?;

        Ok(())
    }

    pub fn get_processes_on_ports_verbose(ports: &[u16], args: &Args) -> (usize, HashMap<u16, crate::types::ProcessInfo>) {
        use crate::process_monitor::ProcessMonitor;
        use crossbeam_channel::bounded;
        use std::collections::HashMap;
        
        // Create a temporary ProcessMonitor to get verbose information
        let (update_sender, _update_receiver) = bounded(100);
        if let Ok(process_monitor) = ProcessMonitor::new(update_sender, ports.to_vec(), args.docker, args.verbose) {
            // Use tokio runtime to run the async scan_processes method
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(process_monitor.scan_processes()) {
                Ok(processes) => (processes.len(), processes),
                Err(_) => (0, HashMap::new())
            }
        } else {
            (0, HashMap::new())
        }
    }

    pub fn get_processes_on_ports(ports: &[u16], args: &Args) -> (usize, HashMap<u16, crate::types::ProcessInfo>) {
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
                        processes.insert(port, crate::types::ProcessInfo {
                            pid,
                            port,
                            command,
                            name,
                            container_id: None,
                            container_name: None,
                            command_line: None,
                            working_directory: None,
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

    pub fn kill_all_processes(ports: &[u16], args: &Args) -> Result<()> {
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
            match Self::kill_process(pid) {
                Ok(_) => info!("Successfully killed process PID: {}", pid),
                Err(e) => error!("Failed to kill process {}: {}", pid, e),
            }
        }
        
        info!("Finished killing all processes");
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
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

    #[cfg(target_os = "windows")]
    fn kill_process(pid: i32) -> Result<()> {
        use std::process::Command;
        
        info!("Killing process PID: {} on Windows", pid);
        
        // Use taskkill to terminate the process
        let output = Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .output();
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Successfully killed process PID: {}", pid);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to kill process PID {}: {}", pid, stderr);
                }
            }
            Err(e) => {
                warn!("Failed to execute taskkill for PID {}: {}", pid, e);
            }
        }
        
        Ok(())
    }

    pub fn kill_single_process(pid: i32, args: &Args) -> Result<()> {
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
        Self::kill_process(pid)
    }

    /// Check if a process is still running by its PID
    fn is_process_still_running(pid: i32) -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            // On Unix-like systems, use ps to check if process exists
            std::process::Command::new("ps")
                .args(&["-p", &pid.to_string()])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
        
        #[cfg(target_os = "windows")]
        {
            // On Windows, use tasklist to check if process exists
            std::process::Command::new("tasklist")
                .args(&["/FI", &format!("PID eq {}", pid)])
                .output()
                .map(|output| {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.contains(&pid.to_string())
                })
                .unwrap_or(false)
        }
    }
}
