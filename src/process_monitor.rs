use crate::types::{ProcessInfo, ProcessUpdate, ProcessHistory, ProcessHistoryEntry};
use crate::smart_filter::{SmartFilter, FilterStats};
use crate::system_monitor::SystemMonitor;
use anyhow::{Context, Result};
use crossbeam_channel::Sender;
use log::{error, info, warn};
#[cfg(not(target_os = "windows"))]
use nix::sys::signal::{kill, Signal};
#[cfg(not(target_os = "windows"))]
use nix::unistd::Pid;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

const MONITORING_INTERVAL: Duration = Duration::from_secs(2);

pub struct ProcessMonitor {
    update_sender: Sender<ProcessUpdate>,
    current_processes: HashMap<u16, ProcessInfo>,
    ports_to_monitor: Vec<u16>,
    docker_enabled: bool,
    verbose: bool,
    history: ProcessHistory,
    smart_filter: Option<SmartFilter>,
    system_monitor: SystemMonitor,
    performance_enabled: bool,
}

impl ProcessMonitor {
    pub fn new(update_sender: Sender<ProcessUpdate>, ports_to_monitor: Vec<u16>, docker_enabled: bool, verbose: bool) -> Result<Self> {
        Ok(Self {
            update_sender,
            current_processes: HashMap::new(),
            ports_to_monitor,
            docker_enabled,
            verbose,
            history: ProcessHistory::load_from_file(&ProcessHistory::get_history_file_path(), 100).unwrap_or_else(|_| ProcessHistory::new(100)),
            smart_filter: None,
            system_monitor: SystemMonitor::new(),
            performance_enabled: false,
        })
    }

    pub fn new_with_filter(
        update_sender: Sender<ProcessUpdate>, 
        ports_to_monitor: Vec<u16>, 
        docker_enabled: bool, 
        verbose: bool,
        smart_filter: SmartFilter,
    ) -> Result<Self> {
        Ok(Self {
            update_sender,
            current_processes: HashMap::new(),
            ports_to_monitor,
            docker_enabled,
            verbose,
            history: ProcessHistory::load_from_file(&ProcessHistory::get_history_file_path(), 100).unwrap_or_else(|_| ProcessHistory::new(100)),
            smart_filter: Some(smart_filter),
            system_monitor: SystemMonitor::new(),
            performance_enabled: false,
        })
    }

    pub fn new_with_performance(
        update_sender: Sender<ProcessUpdate>, 
        ports_to_monitor: Vec<u16>, 
        docker_enabled: bool, 
        verbose: bool,
        smart_filter: Option<SmartFilter>,
        performance_enabled: bool,
    ) -> Result<Self> {
        Ok(Self {
            update_sender,
            current_processes: HashMap::new(),
            ports_to_monitor,
            docker_enabled,
            verbose,
            history: ProcessHistory::load_from_file(&ProcessHistory::get_history_file_path(), 100).unwrap_or_else(|_| ProcessHistory::new(100)),
            smart_filter,
            system_monitor: SystemMonitor::new(),
            performance_enabled,
        })
    }

    pub async fn start_monitoring(&mut self) -> Result<()> {
        let port_description = if self.ports_to_monitor.len() <= 10 {
            format!("ports: {}", self.ports_to_monitor.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "))
        } else {
            format!("{} ports: {} to {}", 
                self.ports_to_monitor.len(), 
                self.ports_to_monitor.first().unwrap_or(&0), 
                self.ports_to_monitor.last().unwrap_or(&0))
        };
        
        info!("Starting process monitoring on {}", port_description);

        loop {
            match self.scan_processes().await {
                Ok(processes) => {
                    let update = ProcessUpdate::new(processes.clone());
                    
                    // Check if there are any changes
                    if self.current_processes != processes {
                        info!("Process update: {} processes found", update.count);
                        self.current_processes = processes;
                        
                        if let Err(e) = self.update_sender.send(update) {
                            error!("Failed to send process update: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to scan processes: {}", e);
                }
            }

            sleep(MONITORING_INTERVAL).await;
        }
    }

    pub async fn scan_processes(&mut self) -> Result<HashMap<u16, ProcessInfo>> {
        let mut processes = HashMap::new();

        // Refresh system information for performance metrics
        if self.performance_enabled {
            self.system_monitor.refresh();
        }

        for &port in &self.ports_to_monitor {
            if let Ok(mut process_info) = self.get_process_on_port(port).await {
                // Add performance metrics if enabled
                if self.performance_enabled {
                    if let Some(cpu_usage) = self.system_monitor.get_process_cpu_usage(process_info.pid) {
                        process_info.cpu_usage = Some(cpu_usage);
                    }
                    
                    if let Some((memory_bytes, memory_percentage)) = self.system_monitor.get_process_memory_usage(process_info.pid) {
                        process_info.memory_usage = Some(memory_bytes);
                        process_info.memory_percentage = Some(memory_percentage);
                    }
                }
                
                processes.insert(port, process_info);
            }
        }

        // Clean up old processes from system monitor
        if self.performance_enabled {
            self.system_monitor.cleanup_old_processes();
        }

        // Apply smart filtering if enabled
        if let Some(ref filter) = self.smart_filter {
            filter.filter_processes(&mut processes);
        }

        Ok(processes)
    }

    async fn get_process_on_port(&self, port: u16) -> Result<ProcessInfo> {
        #[cfg(target_os = "windows")]
        {
            // Windows: Use netstat to find processes listening on the port
            let output = Command::new("netstat")
                .args(&["-ano"])
                .output()
                .context("Failed to execute netstat command")?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        // Extract port from local address (e.g., "0.0.0.0:3000")
                        if let Some(port_str) = parts[1].split(':').last() {
                            if let Ok(found_port) = port_str.parse::<u16>() {
                                if found_port == port {
                                    if let Ok(pid) = parts[4].parse::<i32>() {
                                        // Get process details
                                        let process_info = self.get_process_details_windows(pid, port).await?;
                                        return Ok(process_info);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Unix-like systems: Use lsof to find processes listening on the port
            let output = Command::new("lsof")
                .args(&["-ti", &format!(":{}", port), "-sTCP:LISTEN"])
                .output()
                .context("Failed to execute lsof command")?;

            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let pid_str = output_str.trim();
                if !pid_str.is_empty() {
                    let pid: i32 = pid_str.parse().context("Failed to parse PID")?;
                    
                    // Get process details using ps
                    let process_info = self.get_process_details(pid, port).await?;
                    return Ok(process_info);
                }
            }
        }

        Err(anyhow::anyhow!("No process found on port {}", port))
    }

    #[cfg(not(target_os = "windows"))]
    async fn get_process_details(&self, pid: i32, port: u16) -> Result<ProcessInfo> {
        // Get process command and name using ps
        let output = Command::new("ps")
            .args(&["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .context("Failed to execute ps command")?;

        let command = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            "unknown".to_string()
        };

        // Extract process name (basename of command)
        let name = command
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string();

        // Get full command line and working directory only if verbose mode is enabled
        let (command_line, working_directory) = if self.verbose {
            log::debug!("ProcessMonitor: Gathering verbose info for PID {} (verbose={})", pid, self.verbose);
            self.get_process_verbose_info(pid).await
        } else {
            (None, None)
        };

        // Check if this process is running in a Docker container (Unix-like systems only)
        let (container_id, container_name) = if self.docker_enabled {
            let (container_id, container_name) = self.get_docker_container_info(pid).await;
            // If no container found, mark as host process
            if container_id.is_none() {
                (Some("host-process".to_string()), Some("Host Process".to_string()))
            } else {
                (container_id, container_name)
            }
        } else {
            (None, None)
        };

        log::debug!("Creating ProcessInfo for PID {} on port {} with command_line: {:?}, working_directory: {:?}", pid, port, command_line, working_directory);
        
        // Create ProcessInfo with basic fields
        let mut process_info = ProcessInfo {
            pid,
            port,
            command,
            name,
            container_id,
            container_name,
            command_line: command_line,
            working_directory: working_directory,
            process_group: None,
            project_name: None,
            cpu_usage: None,
            memory_usage: None,
            memory_percentage: None,
        };
        
        // Determine process group and project name
        process_info.process_group = process_info.determine_process_group();
        process_info.project_name = process_info.extract_project_name();
        
        // Enhance process name with better context
        process_info.name = Self::enhance_process_name(&process_info);
        
        Ok(process_info)
    }

    #[cfg(target_os = "windows")]
    async fn get_process_details_windows(&self, pid: i32, port: u16) -> Result<ProcessInfo> {
        // Get process name using tasklist
        let output = Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .output()
            .context("Failed to execute tasklist command")?;

        let command = if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Parse CSV format: "process.exe","PID","Session Name","Session#","Mem Usage"
                if let Some(name_part) = line.split(',').next() {
                    // Remove quotes and .exe extension
                    let name = name_part.trim_matches('"');
                    if let Some(name_without_ext) = name.strip_suffix(".exe") {
                        // Get verbose information only if verbose mode is enabled
                        let (command_line, working_directory) = if self.verbose {
                            self.get_process_verbose_info_windows(pid).await
                        } else {
                            (None, None)
                        };
                        
                        log::debug!("Creating ProcessInfo (Windows) for PID {} on port {} with command_line: {:?}, working_directory: {:?}", pid, port, command_line, working_directory);
                        
                        let mut process_info = ProcessInfo {
                            pid,
                            port,
                            command: name.to_string(),
                            name: name_without_ext.to_string(),
                            container_id: None,
                            container_name: None,
                            command_line: command_line,
                            working_directory: working_directory,
                            process_group: None,
                            project_name: None,
                            cpu_usage: None,
                            memory_usage: None,
                            memory_percentage: None,
                        };
                        
                        // Determine process group and project name
                        process_info.process_group = process_info.determine_process_group();
                        process_info.project_name = process_info.extract_project_name();
                        
                        return Ok(process_info);
                    }
                    // Get verbose information only if verbose mode is enabled
                    let (command_line, working_directory) = if self.verbose {
                        self.get_process_verbose_info_windows(pid).await
                    } else {
                        (None, None)
                    };
                    
                    log::debug!("Creating ProcessInfo (Windows fallback) for PID {} on port {} with command_line: {:?}, working_directory: {:?}", pid, port, command_line, working_directory);
                    
                    let mut process_info = ProcessInfo {
                        pid,
                        port,
                        command: name.to_string(),
                        name: name.to_string(),
                        container_id: None,
                        container_name: None,
                        command_line: command_line,
                        working_directory: working_directory,
                        process_group: None,
                        project_name: None,
                        cpu_usage: None,
                        memory_usage: None,
                        memory_percentage: None,
                    };
                    
                    // Determine process group and project name
                    process_info.process_group = process_info.determine_process_group();
                    process_info.project_name = process_info.extract_project_name();
                    
                    return Ok(process_info);
                }
            }
            "unknown".to_string()
        } else {
            "unknown".to_string()
        };

        // For Windows, Docker container detection is more complex
        // For now, we'll skip it and focus on basic process detection
        let (container_id, container_name) = if self.docker_enabled {
            // TODO: Implement Windows Docker container detection
            (None, None)
        } else {
            (None, None)
        };

        // Get verbose information
        let (command_line, working_directory) = self.get_process_verbose_info_windows(pid).await;

        log::debug!("Creating ProcessInfo (Windows final) for PID {} on port {} with command_line: {:?}, working_directory: {:?}", pid, port, command_line, working_directory);
        
        let mut process_info = ProcessInfo {
            pid,
            port,
            command: command.clone(),
            name: command,
            container_id: container_id,
            container_name: container_name,
            command_line: command_line,
            working_directory: working_directory,
            process_group: None,
            project_name: None,
            cpu_usage: None,
            memory_usage: None,
            memory_percentage: None,
        };
        
        // Determine process group and project name
        process_info.process_group = process_info.determine_process_group();
        process_info.project_name = process_info.extract_project_name();
        
        // Enhance process name with better context
        process_info.name = Self::enhance_process_name(&process_info);
        
        Ok(process_info)
    }

    #[cfg(not(target_os = "windows"))]
    async fn get_process_verbose_info(&self, pid: i32) -> (Option<String>, Option<String>) {
        let mut command_line = None;
        let mut working_directory = None;

                // Get full command line using ps
                if let Ok(output) = Command::new("ps")
                    .args(&["-p", &pid.to_string(), "-o", "args="])
                    .output()
                {
                    if output.status.success() {
                        let cmd = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !cmd.is_empty() && cmd != "COMMAND" {
                            // Truncate the command to show only the important parts
                            let truncated_cmd = Self::truncate_command_line(&cmd);
                            log::debug!("Verbose info for PID {}: command_line = {}", pid, truncated_cmd);
                            command_line = Some(truncated_cmd);
                        }
                    }
                }

        // Get working directory using lsof
        if let Ok(output) = Command::new("lsof")
            .args(&["-p", &pid.to_string(), "-d", "cwd", "-F", "n"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut found_pid = false;
                let mut found_cwd = false;
                for line in stdout.lines() {
                    if line.starts_with('p') && line[1..] == pid.to_string() {
                        found_pid = true;
                        log::debug!("Found PID {} in lsof output", pid);
                    } else if found_pid && line == "fcwd" {
                        found_cwd = true;
                        log::debug!("Found fcwd for PID {}", pid);
                    } else if found_pid && found_cwd && line.starts_with('n') {
                        let dir = &line[1..]; // Remove the 'n' prefix
                        log::debug!("Found directory line for PID {}: '{}'", pid, dir);
                        if !dir.is_empty() && dir != "/" {
                            // Truncate the directory path to show only the last part
                            let truncated_dir = Self::truncate_directory_path(dir);
                            working_directory = Some(truncated_dir);
                            log::debug!("Verbose info for PID {}: working_directory = {}", pid, dir);
                        } else {
                            log::debug!("Directory '{}' is empty or root, skipping", dir);
                        }
                        break;
                    } else if found_pid && line.starts_with('p') && line[1..] != pid.to_string() {
                        // We've moved to a different PID, reset
                        found_pid = false;
                        found_cwd = false;
                    }
                }
            } else {
                log::debug!("lsof failed for PID {}: {}", pid, String::from_utf8_lossy(&output.stderr));
            }
        } else {
            log::debug!("Failed to run lsof for PID {}", pid);
        }

        (command_line, working_directory)
    }

    fn truncate_command_line(cmd: &str) -> String {
        // Split the command into parts
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return cmd.to_string();
        }

        // Get the executable name (last part of the path)
        let executable = parts[0].split('/').last().unwrap_or(parts[0]);
        
        // If there are arguments, include them
        if parts.len() > 1 {
            let args = &parts[1..];
            format!("{} {}", executable, args.join(" "))
        } else {
            executable.to_string()
        }
    }

    fn truncate_directory_path(dir: &str) -> String {
        // Split the path and get the last two parts (username/project)
        let parts: Vec<&str> = dir.split('/').collect();
        if parts.len() >= 2 {
            // Get the last two parts: username/project
            let last_two = &parts[parts.len()-2..];
            last_two.join("/")
        } else if parts.len() == 1 && !parts[0].is_empty() {
            // Just one part, use it as is
            parts[0].to_string()
        } else {
            // Fallback to original path
            dir.to_string()
        }
    }

    #[cfg(target_os = "windows")]
    async fn get_process_verbose_info_windows(&self, pid: i32) -> (Option<String>, Option<String>) {
        let mut command_line = None;
        let mut working_directory = None;

        // Get command line using wmic
        if let Ok(output) = Command::new("wmic")
            .args(&["process", "where", &format!("ProcessId={}", pid), "get", "CommandLine", "/format:list"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.starts_with("CommandLine=") {
                        let cmd = &line[12..]; // Remove "CommandLine=" prefix
                        if !cmd.is_empty() && cmd != "CommandLine" {
                            // Truncate the command to show only the important parts
                            let truncated_cmd = Self::truncate_command_line(cmd);
                            command_line = Some(truncated_cmd);
                        }
                        break;
                    }
                }
            }
        }

        // Get working directory using wmic
        if let Ok(output) = Command::new("wmic")
            .args(&["process", "where", &format!("ProcessId={}", pid), "get", "ExecutablePath", "/format:list"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.starts_with("ExecutablePath=") {
                        let path = &line[15..]; // Remove "ExecutablePath=" prefix
                        if !path.is_empty() && path != "ExecutablePath" {
                            // Extract directory from full path
                            if let Some(last_slash) = path.rfind('\\') {
                                let dir = &path[..last_slash];
                                if !dir.is_empty() {
                                    working_directory = Some(dir.to_string());
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }

        (command_line, working_directory)
    }

    #[cfg(not(target_os = "windows"))]
    async fn get_docker_container_info(&self, pid: i32) -> (Option<String>, Option<String>) {
        // Try to find the container ID for this PID
        let container_id = match self.find_container_id_for_pid(pid).await {
            Ok(id) => id,
            Err(_) => None,
        };

        // If we found a container ID, get the container name
        let container_name = if let Some(ref id) = container_id {
            match self.get_container_name(id).await {
                Ok(name) => Some(name),
                Err(_) => None,
            }
        } else {
            None
        };

        (container_id, container_name)
    }

    #[cfg(not(target_os = "windows"))]
    async fn find_container_id_for_pid(&self, pid: i32) -> Result<Option<String>> {
        // Use docker ps to get all running containers
        let output = Command::new("docker")
            .args(&["ps", "--format", "table {{.ID}}\t{{.Names}}\t{{.Ports}}"])
            .output()
            .context("Failed to execute docker ps command")?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let container_id = parts[0].trim();
                let _ports_str = parts[2].trim();
                
                // Check if this container is using the port we're interested in
                if self.container_has_pid(container_id, pid).await? {
                    return Ok(Some(container_id.to_string()));
                }
            }
        }

        Ok(None)
    }

    #[cfg(not(target_os = "windows"))]
    async fn container_has_pid(&self, container_id: &str, pid: i32) -> Result<bool> {
        // Use docker top to get processes in the container
        let output = Command::new("docker")
            .args(&["top", container_id])
            .output()
            .context("Failed to execute docker top command")?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check if the PID exists in the container's process list
        for line in stdout.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(container_pid) = parts[1].parse::<i32>() {
                    if container_pid == pid {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    #[cfg(not(target_os = "windows"))]
    async fn get_container_name(&self, container_id: &str) -> Result<String> {
        // Get container name using docker inspect
        let output = Command::new("docker")
            .args(&["inspect", "--format", "{{.Name}}", container_id])
            .output()
            .context("Failed to execute docker inspect command")?;

        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Remove leading slash if present
            Ok(name.trim_start_matches('/').to_string())
        } else {
            Ok(container_id.to_string())
        }
    }

    pub async fn kill_process(&mut self, pid: i32) -> Result<()> {
        self.kill_process_with_context(pid, "user", true).await
    }
    
    pub async fn kill_process_with_context(&mut self, pid: i32, context: &str, add_to_history: bool) -> Result<()> {
        info!("Attempting to kill process {}", pid);
        
        // Find the process info before killing it
        let process_info = self.current_processes.values()
            .find(|p| p.pid == pid)
            .cloned();

        #[cfg(not(target_os = "windows"))]
        {
            // Check if this is a Docker container process (Unix-like systems only)
            if self.docker_enabled {
                if let Some(container_id) = self.find_container_id_for_pid(pid).await? {
                    info!("Process {} is in Docker container {}, stopping container", pid, container_id);
                    return self.stop_docker_container(&container_id).await;
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: Use taskkill
            let output = Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .output()
                .context("Failed to execute taskkill command")?;

            if output.status.success() {
                info!("Successfully killed process {} on Windows", pid);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Failed to kill process {} on Windows: {}", pid, stderr);
                return Err(anyhow::anyhow!("Failed to kill process on Windows: {}", stderr));
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Unix-like systems: Use SIGTERM then SIGKILL
            match kill(Pid::from_raw(pid), Signal::SIGTERM) {
                Ok(_) => {
                    info!("Sent SIGTERM to process {}", pid);
                    
                    // Wait a bit and check if process is still alive
                    sleep(Duration::from_millis(500)).await;
                    
                    // Check if process is still running
                    if self.is_process_running(pid).await {
                        warn!("Process {} still running after SIGTERM, sending SIGKILL", pid);
                        
                        // Send SIGKILL if process is still alive
                        match kill(Pid::from_raw(pid), Signal::SIGKILL) {
                            Ok(_) => {
                                info!("Sent SIGKILL to process {}", pid);
                            }
                            Err(e) => {
                                error!("Failed to send SIGKILL to process {}: {}", pid, e);
                                return Err(anyhow::anyhow!("Failed to kill process: {}", e));
                            }
                        }
                    } else {
                        info!("Process {} terminated successfully with SIGTERM", pid);
                    }
                }
                Err(e) => {
                    error!("Failed to send SIGTERM to process {}: {}", pid, e);
                    return Err(anyhow::anyhow!("Failed to kill process: {}", e));
                }
            }
        }
        
        // Add to history if we found the process info and add_to_history is true
        if add_to_history {
            if let Some(process_info) = process_info {
                let history_entry = ProcessHistoryEntry::new(&process_info, context.to_string());
                self.history.add_entry(history_entry);
                info!("Added process {} to history", pid);
                
                // Save history to file
                if let Err(e) = self.history.save_to_file(&ProcessHistory::get_history_file_path()) {
                    warn!("Failed to save history to file: {}", e);
                }
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    async fn stop_docker_container(&self, container_id: &str) -> Result<()> {
        info!("Stopping Docker container: {}", container_id);

        // First try graceful stop
        let stop_output = Command::new("docker")
            .args(&["stop", container_id])
            .output()
            .context("Failed to execute docker stop command")?;

        if stop_output.status.success() {
            info!("Docker container {} stopped gracefully", container_id);
            return Ok(());
        }

        // If graceful stop failed, try force remove
        info!("Graceful stop failed, force removing container: {}", container_id);
        let remove_output = Command::new("docker")
            .args(&["rm", "-f", container_id])
            .output()
            .context("Failed to execute docker rm command")?;

        if remove_output.status.success() {
            info!("Docker container {} force removed", container_id);
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&remove_output.stderr);
            Err(anyhow::anyhow!("Failed to remove Docker container {}: {}", container_id, error_msg))
        }
    }

    pub async fn kill_all_processes(&mut self) -> Result<()> {
        info!("Killing all monitored processes");

        let processes = self.scan_processes().await?;
        let mut errors = Vec::new();

        for (port, process_info) in processes {
            info!("Killing process on port {} (PID: {})", port, process_info.pid);
            
            // Add to history before killing
            let history_entry = ProcessHistoryEntry::new(&process_info, "bulk".to_string());
            self.history.add_entry(history_entry);
            
            if let Err(e) = self.kill_process_with_context(process_info.pid, "bulk", false).await {
                errors.push(format!("Port {} (PID {}): {}", port, process_info.pid, e));
            }
        }

        if !errors.is_empty() {
            let error_msg = errors.join("; ");
            return Err(anyhow::anyhow!("Some processes failed to kill: {}", error_msg));
        }

        // Save history to file after killing all processes
        if let Err(e) = self.history.save_to_file(&ProcessHistory::get_history_file_path()) {
            warn!("Failed to save history to file: {}", e);
        }

        info!("All processes killed successfully");
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    async fn is_process_running(&self, pid: i32) -> bool {
        let output = Command::new("ps")
            .args(&["-p", &pid.to_string()])
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    pub fn get_history(&self) -> &ProcessHistory {
        &self.history
    }
    
    pub fn get_recent_history(&self, limit: usize) -> &[ProcessHistoryEntry] {
        self.history.get_recent_entries(limit)
    }
    
    pub fn clear_history(&mut self) {
        self.history.clear();
        // Save empty history to file
        if let Err(e) = self.history.save_to_file(&ProcessHistory::get_history_file_path()) {
            warn!("Failed to save cleared history to file: {}", e);
        }
    }
    
    pub fn get_filter_stats(&self) -> Option<FilterStats> {
        self.smart_filter.as_ref().map(|filter| filter.get_filter_stats())
    }

    /// Get current processes
    pub fn get_processes(&self) -> &HashMap<u16, ProcessInfo> {
        &self.current_processes
    }

    /// Get ports to monitor
    pub fn get_ports_to_monitor(&self) -> &Vec<u16> {
        &self.ports_to_monitor
    }
    
    /// Enhance process name with better context and descriptions
    fn enhance_process_name(process_info: &ProcessInfo) -> String {
        let original_name = &process_info.name;
        
        // If we have a command line, try to extract a better name from it
        if let Some(ref cmd_line) = process_info.command_line {
            // Look for common patterns in command lines
            if cmd_line.contains("node") && cmd_line.contains("server") {
                return "Node.js Server".to_string();
            } else if cmd_line.contains("python") && cmd_line.contains("app") {
                return "Python App".to_string();
            } else if cmd_line.contains("npm") && cmd_line.contains("start") {
                return "NPM Start".to_string();
            } else if cmd_line.contains("yarn") && cmd_line.contains("start") {
                return "Yarn Start".to_string();
            } else if cmd_line.contains("docker") && cmd_line.contains("run") {
                return "Docker Container".to_string();
            } else if cmd_line.contains("java") && cmd_line.contains("jar") {
                return "Java Application".to_string();
            } else if cmd_line.contains("rails") && cmd_line.contains("server") {
                return "Rails Server".to_string();
            } else if cmd_line.contains("php") && cmd_line.contains("serve") {
                return "PHP Server".to_string();
            } else if cmd_line.contains("go") && cmd_line.contains("run") {
                return "Go Application".to_string();
            } else if cmd_line.contains("rust") && cmd_line.contains("run") {
                return "Rust Application".to_string();
            }
        }
        
        // If we have a working directory, try to infer the purpose
        if let Some(ref work_dir) = process_info.working_directory {
            if work_dir.contains("frontend") || work_dir.contains("client") {
                return format!("{} (Frontend)", original_name);
            } else if work_dir.contains("backend") || work_dir.contains("api") {
                return format!("{} (Backend)", original_name);
            } else if work_dir.contains("database") || work_dir.contains("db") {
                return format!("{} (Database)", original_name);
            } else if work_dir.contains("test") || work_dir.contains("spec") {
                return format!("{} (Test)", original_name);
            }
        }
        
        // If we have a process group, use that for context
        if let Some(ref group) = process_info.process_group {
            match group.as_str() {
                "Node.js" => "Node.js Process".to_string(),
                "Python" => "Python Process".to_string(),
                "Java" => "Java Process".to_string(),
                "Docker" => "Docker Container".to_string(),
                "Web Server" => "Web Server".to_string(),
                "Database" => "Database Server".to_string(),
                _ => original_name.clone(),
            }
        } else {
            original_name.clone()
        }
    }
}

    // Platform-agnostic process management functions
pub fn get_processes_on_ports(ports: &[u16], args: &crate::cli::Args) -> (usize, std::collections::HashMap<u16, crate::types::ProcessInfo>) {
    // If verbose mode is enabled, use ProcessMonitor to get detailed information
    if args.verbose {
        use crossbeam_channel::bounded;
        
        let (update_sender, _update_receiver) = bounded(100);
        if let Ok(mut process_monitor) = ProcessMonitor::new(update_sender, ports.to_vec(), args.docker, args.verbose) {
            // Use block_in_place to avoid runtime conflicts when already in a tokio runtime
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(process_monitor.scan_processes())
            }) {
                Ok(processes) => return (processes.len(), processes),
                Err(e) => {
                    log::warn!("Failed to get verbose process info: {}, falling back to basic mode", e);
                }
            }
        } else {
            log::warn!("Failed to create ProcessMonitor for verbose mode, falling back to basic mode");
        }
    }
    
    // Build lsof command with multiple -i flags for each port
    let mut lsof_args = vec!["-sTCP:LISTEN".to_string(), "-P".to_string(), "-n".to_string()];
    for port in ports {
        lsof_args.push("-i".to_string());
        lsof_args.push(format!(":{}", port));
    }
    
    // Use lsof to get detailed process information
    let output = std::process::Command::new("lsof")
        .args(&lsof_args)
        .output();
        
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut processes = std::collections::HashMap::new();
            
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
                        log::debug!("Creating ProcessInfo (lsof fallback) for PID {} on port {} with command_line: None, working_directory: None", pid, port);
                        
                        let mut process_info = crate::types::ProcessInfo {
                            pid,
                            port,
                            command,
                            name,
                            container_id: None,
                            container_name: None,
                            command_line: None,
                            working_directory: None,
                            process_group: None,
                            project_name: None,
                            cpu_usage: None,
                            memory_usage: None,
                            memory_percentage: None,
                        };
                        
                        // Determine process group and project name
                        process_info.process_group = process_info.determine_process_group();
                        process_info.project_name = process_info.extract_project_name();
                        
                        processes.insert(port, process_info);
                        } else {
                            log::info!("Ignoring process {} (PID {}) on port {} (ignored by user configuration)", name, pid, port);
                        }
                    }
                }
            }
            
            (processes.len(), processes)
        }
        Err(_) => (0, std::collections::HashMap::new())
    }
}

#[cfg(target_os = "windows")]
pub fn kill_all_processes(ports: &[u16], args: &crate::cli::Args) -> anyhow::Result<()> {
    let port_list = ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
    log::info!("Killing all processes on ports {}...", port_list);
    
    // Get ignore sets for efficient lookup
    let ignore_ports = args.get_ignore_ports_set();
    let ignore_processes = args.get_ignore_processes_set();
    
    let mut pids_to_kill = Vec::new();
    
    // On Windows, use netstat to find processes on ports
    for &port in ports {
        if ignore_ports.contains(&port) {
            log::info!("Ignoring port {} during kill operation (ignored by user configuration)", port);
            continue;
        }
        
        let output = match std::process::Command::new("netstat")
            .args(&["-ano", "-p", "TCP"])
            .output() {
            Ok(output) => output,
            Err(e) => {
                log::error!("Failed to run netstat command: {}", e);
                continue;
            }
        };
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pid_str) = parts.last() {
                    if let Ok(pid) = pid_str.parse::<i32>() {
                        if !pids_to_kill.contains(&pid) {
                            pids_to_kill.push(pid);
                        }
                    }
                }
            }
        }
    }
    
    if pids_to_kill.is_empty() {
        log::info!("No processes found to kill on the specified ports");
        return Ok(());
    }
    
    log::info!("Found {} processes to kill", pids_to_kill.len());
    
    for pid in pids_to_kill {
        log::info!("Attempting to kill process PID: {}", pid);
        match kill_process(pid) {
            Ok(_) => log::info!("Successfully killed process PID: {}", pid),
            Err(e) => log::error!("Failed to kill process {}: {}", pid, e),
        }
    }
    
    log::info!("Finished killing all processes");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn kill_all_processes(ports: &[u16], args: &crate::cli::Args) -> anyhow::Result<()> {
    // Build port range string for lsof
    let port_list = ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
    log::info!("Killing all processes on ports {}...", port_list);
    
    // Build lsof command with multiple -i flags for each port
    let mut lsof_args = vec!["-sTCP:LISTEN".to_string(), "-P".to_string(), "-n".to_string()];
    for port in ports {
        lsof_args.push("-i".to_string());
        lsof_args.push(format!(":{}", port));
    }
    
    // Get all PIDs on the monitored ports
    let output = match std::process::Command::new("lsof")
        .args(&lsof_args)
        .output() {
        Ok(output) => output,
        Err(e) => {
            log::error!("Failed to run lsof command: {}", e);
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
                    log::info!("Ignoring process {} (PID {}) on port {} during kill operation (ignored by user configuration)", name, pid, port);
                }
            }
        }
    }
    
    if pids_to_kill.is_empty() {
        log::info!("No processes found to kill (all were ignored or none found)");
        return Ok(());
    }
    
    log::info!("Found {} processes to kill (after filtering ignored processes)", pids_to_kill.len());
    
    for pid in pids_to_kill {
        log::info!("Attempting to kill process PID: {}", pid);
        match kill_process(pid) {
            Ok(_) => log::info!("Successfully killed process PID: {}", pid),
            Err(e) => log::error!("Failed to kill process {}: {}", pid, e),
        }
    }
    
    log::info!("Finished killing all processes");
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn kill_single_process(pid: i32, _args: &crate::cli::Args) -> anyhow::Result<()> {
    log::info!("Killing single process PID: {}", pid);
    // On Windows, simplified version - just kill the process
    // Process filtering is done at a higher level
    kill_process(pid)
}

#[cfg(not(target_os = "windows"))]
pub fn kill_single_process(pid: i32, args: &crate::cli::Args) -> anyhow::Result<()> {
    log::info!("Killing single process PID: {}", pid);
    
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
            log::info!("Ignoring process {} (PID {}) - process name is in ignore list", process_name, pid);
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
                        log::info!("Ignoring process on port {} (PID {}) - port is in ignore list", port, pid);
                        return Ok(());
                    }
                }
            }
        }
    }
    
    // Process is not ignored, proceed with killing
    kill_process(pid)
}

fn kill_process(pid: i32) -> anyhow::Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        
        log::info!("Killing process PID: {} with SIGTERM", pid);
        
        // First try SIGTERM (graceful termination)
        match kill(Pid::from_raw(pid), Signal::SIGTERM) {
            Ok(_) => log::info!("SIGTERM sent to PID: {}", pid),
            Err(e) => {
                // Don't fail immediately, just log the error and continue
                log::warn!("Failed to send SIGTERM to PID {}: {} (process may already be terminated)", pid, e);
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
            log::info!("Process {} still running, sending SIGKILL", pid);
            match kill(Pid::from_raw(pid), Signal::SIGKILL) {
                Ok(_) => log::info!("SIGKILL sent to PID: {}", pid),
                Err(e) => {
                    // Log error but don't fail the entire operation
                    log::warn!("Failed to send SIGKILL to PID {}: {} (process may be protected)", pid, e);
                }
            }
        } else {
            log::info!("Process {} terminated gracefully", pid);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        log::info!("Killing process PID: {} on Windows", pid);
        
        // Use taskkill to terminate the process
        let output = Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .output();
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    log::info!("Successfully killed process PID: {}", pid);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!("Failed to kill process PID {}: {}", pid, stderr);
                }
            }
            Err(e) => {
                log::warn!("Failed to execute taskkill for PID {}: {}", pid, e);
            }
        }
    }
    
    Ok(())
}
