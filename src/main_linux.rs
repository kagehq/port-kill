// Linux-specific main entry point
// This provides Linux tray support while maintaining all core functionality

use cli::Args;
use console_app::ConsolePortKillApp;
use types::{ProcessInfo, StatusBarInfo};
use process_monitor::{get_processes_on_ports, kill_all_processes, kill_single_process};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::env;
use std::process;
use std::collections::HashMap;
use std::time::Duration;
use std::rc::Rc;
use std::cell::RefCell;

// GTK initialization for tray support
use gtk::prelude::*;
use gtk::{Menu, MenuItem, SeparatorMenuItem};

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
    
    // Initialize GTK before creating tray items
    if gtk::init().is_err() {
        return Err(anyhow::anyhow!("Failed to initialize GTK"));
    }
    info!("GTK initialized successfully");
    
    // Create the app indicator (tray icon)
    let indicator = Rc::new(RefCell::new(AppIndicator::new("port-kill", "port-kill")));
    indicator.borrow_mut().set_status(AppIndicatorStatus::Active);
    
    // Set initial icon based on process count
    let (initial_count, _) = get_processes_on_ports(&args.get_ports_to_monitor(), &args);
    update_tray_icon(&mut indicator.borrow_mut(), initial_count);
    
    // Create the main menu
    let mut menu = Menu::new();
    
    // Add status header
    let status_item = MenuItem::with_label(&format!("Port Status: {} processes", initial_count));
    status_item.set_sensitive(false); // Make it non-clickable
    menu.append(&status_item);
    
    // Add separator
    let separator = SeparatorMenuItem::new();
    menu.append(&separator);
    
    // Add process-specific submenu (will be updated dynamically)
        let process_menu = create_process_menu_with_verbose(&args, &HashMap::new(), args.verbose);
    let process_root = MenuItem::with_label("Processes");
    process_root.set_submenu(Some(&process_menu));
    menu.append(&process_root);
    
    // Add another separator
    let separator2 = SeparatorMenuItem::new();
    menu.append(&separator2);
    
    // Add action items
    let kill_all_item = MenuItem::with_label("Kill All Processes");
    let args_clone = args.clone();
    kill_all_item.connect_activate(move |_| {
        info!("Kill All Processes clicked");
        let ports_to_kill = args_clone.get_ports_to_monitor();
        if let Err(e) = kill_all_processes(&ports_to_kill, &args_clone) {
            error!("Failed to kill all processes: {}", e);
        }
    });
    menu.append(&kill_all_item);
    
    let quit_item = MenuItem::with_label("Quit");
    quit_item.connect_activate(move |_| {
        info!("Quit clicked, exiting gracefully...");
        process::exit(0);
    });
    menu.append(&quit_item);
    
    // Set the menu on the indicator
    indicator.borrow_mut().set_menu(&mut menu);
    
    info!("Enhanced tray icon created successfully!");
    println!("🔍 Look for the Port Kill icon in your system tray!");
    println!("💡 Features: Dynamic process menu, status display, individual process control");
    
    // Set up periodic updates using GTK timeout
    let args_clone = args.clone();
    let indicator_clone = indicator.clone();
    gtk::glib::timeout_add_local(Duration::from_secs(5), move || {
        // Get current processes
        let (process_count, processes) = 
            get_processes_on_ports(&args_clone.get_ports_to_monitor(), &args_clone);
        
        // Update tray icon
        if let Ok(mut ind) = indicator_clone.try_borrow_mut() {
            update_tray_icon(&mut ind, process_count);
        }
        
        // Update status display
        let status_info = StatusBarInfo::from_process_count(process_count);
        println!("🔄 Port Status: {} - {}", status_info.text, status_info.tooltip);
        
        // Print detected processes with grouping
        if process_count > 0 {
            println!("📋 Detected Processes:");
            
            // Group processes by type
            let mut grouped_processes: std::collections::HashMap<String, Vec<(&u16, &ProcessInfo)>> = std::collections::HashMap::new();
            let mut ungrouped_processes = Vec::new();
            
            for (port, process_info) in &processes {
                if let Some(ref group) = process_info.process_group {
                    grouped_processes.entry(group.clone()).or_insert_with(Vec::new).push((port, process_info));
                } else {
                    ungrouped_processes.push((port, process_info));
                }
            }
            
            // Print grouped processes
            for (group_name, group_processes) in &grouped_processes {
                println!("   🔹 {} ({} processes):", group_name, group_processes.len());
                for (port, process_info) in group_processes {
                    let display_name = process_info.get_display_name();
                    if args_clone.verbose {
                        // Verbose mode: show command line and working directory
                        let mut parts = vec![format!("      • Port {}: {}", port, display_name)];
                        
                        if let Some(ref cmd_line) = process_info.command_line {
                            parts.push(format!("({})", cmd_line));
                        }
                        
                        if args_clone.show_pid {
                            parts.push(format!("(PID {})", process_info.pid));
                        }
                        
                        if let Some(ref work_dir) = process_info.working_directory {
                            parts.push(format!("- {}", work_dir));
                        }
                        
                        if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                            parts.push(format!("[Docker: {}]", container_name));
                        }
                        
                        println!("{}", parts.join(" "));
                    } else if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                        println!("      • Port {}: {} [Docker: {}]", port, display_name, container_name);
                    } else if args_clone.show_pid {
                        println!("      • Port {}: {} (PID {})", port, display_name, process_info.pid);
                    } else {
                        println!("      • Port {}: {}", port, display_name);
                    }
                }
            }
            
            // Print ungrouped processes
            if !ungrouped_processes.is_empty() {
                println!("   🔹 Other ({} processes):", ungrouped_processes.len());
                for (port, process_info) in &ungrouped_processes {
                    let display_name = process_info.get_display_name();
                    if args_clone.verbose {
                        // Verbose mode: show command line and working directory
                        let mut parts = vec![format!("      • Port {}: {}", port, display_name)];
                        
                        if let Some(ref cmd_line) = process_info.command_line {
                            parts.push(format!("({})", cmd_line));
                        }
                        
                        if args_clone.show_pid {
                            parts.push(format!("(PID {})", process_info.pid));
                        }
                        
                        if let Some(ref work_dir) = process_info.working_directory {
                            parts.push(format!("- {}", work_dir));
                        }
                        
                        if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                            parts.push(format!("[Docker: {}]", container_name));
                        }
                        
                        println!("{}", parts.join(" "));
                    } else if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                        println!("      • Port {}: {} [Docker: {}]", port, display_name, container_name);
                    } else if args_clone.show_pid {
                        println!("      • Port {}: {} (PID {})", port, display_name, process_info.pid);
                    } else {
                        println!("      • Port {}: {}", port, display_name);
                    }
                }
            }
        } else {
            println!("📋 No processes detected");
        }
        
        // Continue the timeout
        gtk::glib::Continue(true)
    });
    
    info!("Enhanced tray mode started successfully!");
    
    // Start GTK main loop
    gtk::main();
    
    Ok(())
}

/// Create a dynamic menu for processes
fn create_process_menu(args: &Args, processes: &HashMap<u16, ProcessInfo>) -> Menu {
    create_process_menu_with_verbose(args, processes, false)
}

/// Create a dynamic menu for processes with verbose information
fn create_process_menu_with_verbose(args: &Args, processes: &HashMap<u16, ProcessInfo>, verbose: bool) -> Menu {
    let menu = Menu::new();
    
    if processes.is_empty() {
        let no_processes_item = MenuItem::with_label("No processes detected");
        no_processes_item.set_sensitive(false);
        menu.append(&no_processes_item);
        return menu;
    }
    
    // Sort processes by port for consistent ordering
    let mut sorted_processes: Vec<_> = processes.iter().collect();
    sorted_processes.sort_by_key(|(port, _)| *port);
    
    for (port, process_info) in sorted_processes {
        let label = if verbose {
            // Verbose mode: show command line and working directory
            let mut parts = vec![format!("Port {}: {}", port, process_info.name)];
            
            if let Some(ref cmd_line) = process_info.command_line {
                parts.push(format!("({})", cmd_line));
            }
            
            if args.show_pid {
                parts.push(format!("(PID {})", process_info.pid));
            }
            
            if let Some(ref work_dir) = process_info.working_directory {
                parts.push(format!("- {}", work_dir));
            }
            
            parts.join(" ")
        } else if args.show_pid {
            format!("Port {}: {} (PID {})", port, process_info.name, process_info.pid)
        } else {
            format!("Port {}: {}", port, process_info.name)
        };
        
        let menu_item = MenuItem::with_label(&label);
        let port_clone = *port;
        let args_clone = args.clone();
        let pid_to_kill = process_info.pid;
        
        menu_item.connect_activate(move |_| {
            info!("Killing process on port {} (PID: {})", port_clone, pid_to_kill);
            if let Err(e) = kill_single_process(pid_to_kill, &args_clone) {
                error!("Failed to kill process on port {}: {}", port_clone, e);
            } else {
                info!("Successfully killed process on port {}", port_clone);
            }
        });
        
        menu.append(&menu_item);
    }
    
    menu
}

/// Update the tray icon based on process count
fn update_tray_icon(indicator: &mut AppIndicator, process_count: usize) {
    let icon_name = match process_count {
        0 => "port-kill-green",      // Green for no processes
        1..=9 => "port-kill-orange", // Orange for 1-9 processes
        _ => "port-kill-red",        // Red for 10+ processes
    };
    
    indicator.set_icon(icon_name);
    
    // Update tooltip
    let tooltip = match process_count {
        0 => "Port Kill - No processes detected".to_string(),
        1 => "Port Kill - 1 process running".to_string(),
        _ => format!("Port Kill - {} processes running", process_count),
    };
    indicator.set_title(&tooltip);
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
