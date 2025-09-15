use crate::{
    process_monitor::ProcessMonitor,
    types::{ProcessUpdate, StatusBarInfo},
    cli::Args,
    smart_filter::SmartFilter,
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

        // Create smart filter if needed
        let smart_filter = Self::create_smart_filter(&args)?;
        
        // Create process monitor with configurable ports
        println!("DEBUG: Creating ProcessMonitor with verbose={}, performance={}", args.verbose, args.performance);
        let process_monitor = if let Some(filter) = smart_filter {
            Arc::new(Mutex::new(ProcessMonitor::new_with_performance(update_sender, args.get_ports_to_monitor(), args.docker, args.verbose, Some(filter), args.performance)?))
        } else {
            Arc::new(Mutex::new(ProcessMonitor::new_with_performance(update_sender, args.get_ports_to_monitor(), args.docker, args.verbose, None, args.performance)?))
        };

        Ok(Self {
            process_monitor,
            update_receiver,
            args,
        })
    }
    
    fn create_smart_filter(args: &Args) -> Result<Option<SmartFilter>> {
        // Get smart filter defaults
        let (smart_ignore_ports, smart_ignore_processes, smart_ignore_groups) = args.get_smart_filter_defaults();
        
        // Combine user ignores with smart ignores
        let mut ignore_ports = args.get_ignore_ports_set();
        ignore_ports.extend(smart_ignore_ports);
        
        let mut ignore_processes = args.get_ignore_processes_set();
        ignore_processes.extend(smart_ignore_processes);
        
        let mut ignore_groups = args.get_ignore_groups_set();
        ignore_groups.extend(smart_ignore_groups);
        
        // Check if any filtering is needed
        if ignore_ports.is_empty() 
            && ignore_processes.is_empty() 
            && args.ignore_patterns.is_none()
            && ignore_groups.is_empty() 
            && args.only_groups.is_none() {
            return Ok(None);
        }
        
        let filter = SmartFilter::new(
            ignore_ports,
            ignore_processes,
            args.ignore_patterns.clone(),
            ignore_groups,
            args.get_only_groups_set(),
        )?;
        
        Ok(Some(filter))
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Starting Console Port Kill application...");
        
        if self.args.json {
            // JSON mode - output processes once and exit
            return self.output_processes_json().await;
        }
        
        println!("🚀 Port Kill Console Monitor Started!");
        println!("📡 Monitoring {} every 2 seconds...", self.args.get_port_description());
        
        // Show filter information if filtering is enabled
        if let Ok(monitor) = self.process_monitor.try_lock() {
            if let Some(filter_stats) = monitor.get_filter_stats() {
                println!("🔍 {}", filter_stats.get_description());
            }
        }
        
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
    
    async fn output_processes_json(&mut self) -> Result<()> {
        // Directly scan processes instead of using monitoring loop
        let mut monitor = self.process_monitor.lock().await;
        let processes = monitor.scan_processes().await?;
        
        // Filter out ignored processes
        let filtered_processes = self.filter_ignored_processes(&processes);
        
        // Output each process as JSON
        for (_port, process_info) in &filtered_processes {
            let json = serde_json::to_string(process_info)?;
            println!("{}", json);
        }
        
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
                let status_info = StatusBarInfo::from_processes_with_status(&filtered_processes);
                
                // Print status to console
                println!("🔄 Port Status: {} - {}", status_info.text, status_info.tooltip);
                
                if filtered_count > 0 {
                    println!("📋 Detected Processes (after filtering ignored):");
                    
                    // Show process summary
                    let mut group_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                    let mut project_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                    
                    for (_, process_info) in &filtered_processes {
                        if let Some(ref group) = process_info.process_group {
                            *group_counts.entry(group.clone()).or_insert(0) += 1;
                        }
                        if let Some(ref project) = process_info.project_name {
                            *project_counts.entry(project.clone()).or_insert(0) += 1;
                        }
                    }
                    
                    if !group_counts.is_empty() {
                        let group_summary: Vec<String> = group_counts.iter()
                            .map(|(group, count)| format!("{}: {}", group, count))
                            .collect();
                        println!("   📊 Groups: {}", group_summary.join(", "));
                    }
                    
                    if !project_counts.is_empty() {
                        let project_summary: Vec<String> = project_counts.iter()
                            .map(|(project, count)| format!("{}: {}", project, count))
                            .collect();
                        println!("   📁 Projects: {}", project_summary.join(", "));
                    }
                    
                    println!();
                    
                    for (_port, process_info) in &filtered_processes {
                        if self.args.verbose {
                            // Verbose mode: show detailed description
                            let mut parts = vec![format!("   • {}", process_info.get_detailed_description())];
                            
                            // Add project context if requested
                            if self.args.show_context {
                                parts.push(format!("[Context: {}]", process_info.get_project_description()));
                            }
                            
                            // Add performance metrics if available
                            if self.args.performance {
                                if let Some(cpu) = process_info.cpu_usage {
                                    let cpu_indicator = if cpu > 50.0 { "🔥" } else if cpu > 20.0 { "⚠️" } else { "✅" };
                                    parts.push(format!("CPU: {:.1}%{}", cpu, cpu_indicator));
                                }
                                if let Some(memory) = process_info.memory_usage {
                                    let memory_mb = memory as f64 / 1024.0 / 1024.0;
                                    let memory_indicator = if memory_mb > 500.0 { "🔥" } else if memory_mb > 100.0 { "⚠️" } else { "✅" };
                                    parts.push(format!("RAM: {:.1}MB{}", memory_mb, memory_indicator));
                                }
                            }
                            
                            if self.args.show_pid {
                                parts.push(format!("(PID {})", process_info.pid));
                            }
                            
                            println!("{}", parts.join(" "));
                        } else {
                            // Normal mode: show enhanced display name
                            let mut parts = vec![format!("   • Port {}: {}", _port, process_info.get_display_name())];
                            
                            // Add project context if requested
                            if self.args.show_context {
                                parts.push(format!("[Context: {}]", process_info.get_project_description()));
                            }
                            
                            // Add performance metrics if available
                            if self.args.performance {
                                if let Some(cpu) = process_info.cpu_usage {
                                    let cpu_indicator = if cpu > 50.0 { "🔥" } else if cpu > 20.0 { "⚠️" } else { "✅" };
                                    parts.push(format!("CPU: {:.1}%{}", cpu, cpu_indicator));
                                }
                                if let Some(memory) = process_info.memory_usage {
                                    let memory_mb = memory as f64 / 1024.0 / 1024.0;
                                    let memory_indicator = if memory_mb > 500.0 { "🔥" } else if memory_mb > 100.0 { "⚠️" } else { "✅" };
                                    parts.push(format!("RAM: {:.1}MB{}", memory_mb, memory_indicator));
                                }
                            }
                            
                            if self.args.show_pid {
                                parts.push(format!("(PID {})", process_info.pid));
                            }
                            
                            println!("{}", parts.join(" "));
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
    
    pub async fn display_history(&self) -> Result<()> {
        let monitor = self.process_monitor.lock().await;
        let history = monitor.get_history();
        
        if history.is_empty() {
            if self.args.json {
                println!("[]");
            } else {
                println!("📋 No process kill history found");
            }
            return Ok(());
        }
        
        if self.args.json {
            // Output history as JSON
            let recent_entries = history.get_recent_entries(50); // Show last 50 entries for API
            
            for entry in recent_entries.iter() {
                let json = serde_json::to_string(entry)?;
                println!("{}", json);
            }
        } else {
            // Output history in human-readable format
            println!("📋 Process Kill History ({} entries):", history.len());
            println!("{}", "─".repeat(80));
            
            let recent_entries = history.get_recent_entries(20); // Show last 20 entries
            
            for (i, entry) in recent_entries.iter().enumerate() {
                let display_name = entry.get_display_name();
                let time_str = format_time_ago(entry.killed_at);
                
                println!("{:2}. {} (PID {}) on port {} - {} ago", 
                    i + 1, 
                    display_name, 
                    entry.pid, 
                    entry.port, 
                    time_str
                );
                
                if let Some(ref cmd_line) = entry.command_line {
                    println!("    Command: {}", cmd_line);
                }
                
                if let Some(ref work_dir) = entry.working_directory {
                    println!("    Directory: {}", work_dir);
                }
                
                println!("    Killed by: {}", entry.killed_by);
                println!();
            }
        }
        
        Ok(())
    }
    
    pub async fn clear_history(&self) -> Result<()> {
        let mut monitor = self.process_monitor.lock().await;
        monitor.clear_history();
        println!("🗑️  Process kill history cleared");
        Ok(())
    }
    
    pub async fn display_filter_info(&self) -> Result<()> {
        let monitor = self.process_monitor.lock().await;
        
        if let Some(filter_stats) = monitor.get_filter_stats() {
            println!("🔍 Filter Configuration:");
            println!("{}", "─".repeat(50));
            println!("{}", filter_stats.get_description());
            
            if filter_stats.ignore_ports_count > 0 {
                println!("  • Ignoring {} system ports", filter_stats.ignore_ports_count);
            }
            if filter_stats.ignore_processes_count > 0 {
                println!("  • Ignoring {} system processes", filter_stats.ignore_processes_count);
            }
            if filter_stats.ignore_patterns_count > 0 {
                println!("  • Using {} pattern filters", filter_stats.ignore_patterns_count);
            }
            if filter_stats.ignore_groups_count > 0 {
                println!("  • Ignoring {} process groups", filter_stats.ignore_groups_count);
            }
            if filter_stats.only_groups_count > 0 {
                println!("  • Showing only {} process groups", filter_stats.only_groups_count);
            }
        } else {
            println!("🔍 No filtering enabled - showing all processes");
        }
        
        Ok(())
    }
    
    pub async fn kill_by_group(&self, groups: &[String]) -> Result<()> {
        let mut monitor = self.process_monitor.lock().await;
        let processes = monitor.scan_processes().await?;
        
        let mut killed_count = 0;
        let mut total_count = 0;
        
        for (port, process_info) in &processes {
            if let Some(ref group) = process_info.process_group {
                if groups.contains(group) {
                    total_count += 1;
                    println!("🔪 Killing {} (PID {}) on port {} - Group: {}", 
                            process_info.get_short_name(), process_info.pid, port, group);
                    
                    if let Err(e) = monitor.kill_process(process_info.pid).await {
                        println!("❌ Failed to kill {} (PID {}): {}", 
                                process_info.get_short_name(), process_info.pid, e);
                    } else {
                        killed_count += 1;
                    }
                }
            }
        }
        
        if total_count == 0 {
            println!("ℹ️  No processes found in groups: {}", groups.join(", "));
        } else {
            println!("✅ Killed {}/{} processes from groups: {}", 
                    killed_count, total_count, groups.join(", "));
        }
        
        Ok(())
    }
    
    pub async fn kill_by_project(&self, projects: &[String]) -> Result<()> {
        let mut monitor = self.process_monitor.lock().await;
        let processes = monitor.scan_processes().await?;
        
        let mut killed_count = 0;
        let mut total_count = 0;
        
        for (port, process_info) in &processes {
            if let Some(ref project) = process_info.project_name {
                if projects.contains(project) {
                    total_count += 1;
                    println!("🔪 Killing {} (PID {}) on port {} - Project: {}", 
                            process_info.get_short_name(), process_info.pid, port, project);
                    
                    if let Err(e) = monitor.kill_process(process_info.pid).await {
                        println!("❌ Failed to kill {} (PID {}): {}", 
                                process_info.get_short_name(), process_info.pid, e);
                    } else {
                        killed_count += 1;
                    }
                }
            }
        }
        
        if total_count == 0 {
            println!("ℹ️  No processes found in projects: {}", projects.join(", "));
        } else {
            println!("✅ Killed {}/{} processes from projects: {}", 
                    killed_count, total_count, projects.join(", "));
        }
        
        Ok(())
    }
    
    pub async fn kill_all_processes(&self) -> Result<()> {
        let mut monitor = self.process_monitor.lock().await;
        let processes = monitor.scan_processes().await?;
        
        if processes.is_empty() {
            println!("ℹ️  No processes found to kill");
            return Ok(());
        }
        
        let total_count = processes.len();
        println!("🔪 Killing all {} processes...", total_count);
        
        // Use the ProcessMonitor's kill_all_processes method which handles history properly
        monitor.kill_all_processes().await?;
        
        println!("✅ Killed all {} processes", total_count);
        
        Ok(())
    }
    
    pub async fn restart_processes(&self) -> Result<()> {
        let mut monitor = self.process_monitor.lock().await;
        let processes = monitor.scan_processes().await?;
        
        if processes.is_empty() {
            println!("ℹ️  No processes to restart");
            return Ok(());
        }
        
        println!("🔄 Restarting {} processes...", processes.len());
        
        // Kill all processes
        for (port, process_info) in &processes {
            println!("🔪 Killing {} (PID {}) on port {}", 
                    process_info.get_short_name(), process_info.pid, port);
            
            if let Err(e) = monitor.kill_process(process_info.pid).await {
                println!("❌ Failed to kill {} (PID {}): {}", 
                        process_info.get_short_name(), process_info.pid, e);
            }
        }
        
        println!("⏳ Waiting 3 seconds for processes to restart...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        // Check if processes have restarted
        let new_processes = monitor.scan_processes().await?;
        if new_processes.is_empty() {
            println!("ℹ️  No processes detected after restart");
        } else {
            println!("✅ Detected {} processes after restart", new_processes.len());
        }
        
        Ok(())
    }
    
    pub async fn show_process_tree(&self) -> Result<()> {
        let mut monitor = self.process_monitor.lock().await;
        let processes = monitor.scan_processes().await?;
        
        if processes.is_empty() {
            println!("ℹ️  No processes detected");
            return Ok(());
        }
        
        println!("🌳 Process Tree:");
        println!("{}", "─".repeat(60));
        
        // Group processes by project for better visualization
        let mut project_groups: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
        
        for (port, process_info) in &processes {
            let project = process_info.project_name.as_ref()
                .map(|p| p.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            project_groups.entry(project).or_insert_with(Vec::new).push((port, process_info));
        }
        
        for (project, project_processes) in &project_groups {
            println!("📁 Project: {}", project);
            for (port, process_info) in project_processes {
                let group_info = process_info.process_group.as_ref()
                    .map(|g| format!(" ({})", g))
                    .unwrap_or_default();
                
                println!("  ├─ Port {}: {}{} (PID {})", 
                        port, process_info.get_short_name(), group_info, process_info.pid);
                
                if let Some(ref work_dir) = process_info.working_directory {
                    println!("  │  └─ Working Directory: {}", work_dir);
                }
                
                if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                    println!("  │  └─ Docker Container: {}", container_name);
                }
            }
            println!();
        }
        
        Ok(())
    }
}

fn format_time_ago(time: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(time);
    
    let seconds = duration.num_seconds();
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
}
