use crate::{
    process_monitor::ProcessMonitor,
    types::{ProcessUpdate, StatusBarInfo},
    cli::Args,
};
use anyhow::Result;
use crossbeam_channel::{bounded, Receiver};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct ConsolePortKillApp {
    process_monitor: Arc<Mutex<ProcessMonitor>>,
    update_receiver: Receiver<ProcessUpdate>,
    args: Args,
}

impl ConsolePortKillApp {
    pub fn new(args: Args) -> Result<Self> {
        // Create channels for communication
        let (update_sender, update_receiver) = bounded(100);

        // Create process monitor with configurable ports
        println!("DEBUG: Creating ProcessMonitor with verbose={}", args.verbose);
        let process_monitor = Arc::new(Mutex::new(ProcessMonitor::new(update_sender, args.get_ports_to_monitor(), args.docker, args.verbose)?));

        Ok(Self {
            process_monitor,
            update_receiver,
            args,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Starting Console Port Kill application...");
        println!("🚀 Port Kill Console Monitor Started!");
        println!("📡 Monitoring {} every 2 seconds...", self.args.get_port_description());
        println!("💡 Press Ctrl+C to quit");
        println!("");

        // Start process monitoring in background
        let monitor = self.process_monitor.clone();
        tokio::spawn(async move {
            if let Err(e) = monitor.lock().await.start_monitoring().await {
                error!("Process monitoring failed: {}", e);
            }
        });

        // Handle updates in the main thread
        self.handle_console_updates().await;

        Ok(())
    }

    async fn handle_console_updates(&mut self) {
        info!("Starting console update handler...");

        loop {
            // Check for process updates
            if let Ok(update) = self.update_receiver.try_recv() {
                // Filter out ignored processes
                let filtered_processes = self.filter_ignored_processes(&update.processes);
                let filtered_count = filtered_processes.len();
                
                // Update status
                let status_info = StatusBarInfo::from_process_count(filtered_count);
                
                // Print status to console
                println!("🔄 Port Status: {} - {}", status_info.text, status_info.tooltip);
                
                if filtered_count > 0 {
                    println!("📋 Detected Processes (after filtering ignored):");
                    for (port, process_info) in &filtered_processes {
                        if self.args.verbose {
                            // Verbose mode: show command line and working directory
                            let mut parts = vec![format!("   • Port {}: {}", port, process_info.name)];
                            
                            if let Some(ref cmd_line) = process_info.command_line {
                                parts.push(format!("({})", cmd_line));
                            }
                            
                            if self.args.show_pid {
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
                            println!("   • Port {}: {} - {} [Docker: {}]", 
                                    port, process_info.name, process_info.command, container_name);
                        } else if self.args.show_pid {
                            println!("   • Port {}: {} (PID {}) - {}", 
                                    port, process_info.name, process_info.pid, process_info.command);
                        } else {
                            println!("   • Port {}: {} - {}", 
                                    port, process_info.name, process_info.command);
                        }
                    }
                }
                
                // Show ignored processes if any
                let ignored_count = update.processes.len() - filtered_count;
                if ignored_count > 0 {
                    println!("🚫 Ignored {} process(es) based on user configuration", ignored_count);
                }
                
                println!("");
            }

            // Sleep briefly to avoid busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    fn filter_ignored_processes(&self, processes: &HashMap<u16, crate::types::ProcessInfo>) -> HashMap<u16, crate::types::ProcessInfo> {
        let mut filtered = HashMap::new();
        
        // Get ignore sets for efficient lookup
        let ignore_ports = self.args.get_ignore_ports_set();
        let ignore_processes = self.args.get_ignore_processes_set();
        
        for (port, process_info) in processes {
            // Check if this process should be ignored
            let should_ignore = ignore_ports.contains(port) || ignore_processes.contains(&process_info.name);
            
            if !should_ignore {
                filtered.insert(*port, process_info.clone());
            } else {
                info!("Console: Ignoring process {} (PID {}) on port {} (ignored by user configuration)", 
                      process_info.name, process_info.pid, port);
            }
        }
        
        filtered
    }
}
