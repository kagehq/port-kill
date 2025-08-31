use crate::types::{ProcessInfo, ProcessUpdate};
use anyhow::{Context, Result};
use crossbeam_channel::Sender;
use log::{error, info};
#[cfg(not(target_os = "windows"))]
use log::warn;
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
}

impl ProcessMonitor {
    pub fn new(update_sender: Sender<ProcessUpdate>, ports_to_monitor: Vec<u16>, docker_enabled: bool) -> Result<Self> {
        Ok(Self {
            update_sender,
            current_processes: HashMap::new(),
            ports_to_monitor,
            docker_enabled,
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

    async fn scan_processes(&self) -> Result<HashMap<u16, ProcessInfo>> {
        let mut processes = HashMap::new();

        for &port in &self.ports_to_monitor {
            if let Ok(process_info) = self.get_process_on_port(port).await {
                processes.insert(port, process_info);
            }
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

        // Check if this process is running in a Docker container
        let (container_id, container_name) = if self.docker_enabled {
            self.get_docker_container_info(pid).await
        } else {
            (None, None)
        };

        Ok(ProcessInfo {
            pid,
            port,
            command,
            name,
            container_id,
            container_name,
        })
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
                        return Ok(ProcessInfo {
                            pid,
                            port,
                            command: name.to_string(),
                            name: name_without_ext.to_string(),
                            container_id: None,
                            container_name: None,
                        });
                    }
                    return Ok(ProcessInfo {
                        pid,
                        port,
                        command: name.to_string(),
                        name: name.to_string(),
                        container_id: None,
                        container_name: None,
                    });
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

        Ok(ProcessInfo {
            pid,
            port,
            command: command.clone(),
            name: command,
            container_id,
            container_name,
        })
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

    pub async fn kill_process(&self, pid: i32) -> Result<()> {
        info!("Attempting to kill process {}", pid);

        // Check if this is a Docker container process
        if self.docker_enabled {
            if let Some(container_id) = self.find_container_id_for_pid(pid).await? {
                info!("Process {} is in Docker container {}, stopping container", pid, container_id);
                return self.stop_docker_container(&container_id).await;
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

        Ok(())
    }

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

    pub async fn kill_all_processes(&self) -> Result<()> {
        info!("Killing all monitored processes");

        let processes = self.scan_processes().await?;
        let mut errors = Vec::new();

        for (port, process_info) in processes {
            info!("Killing process on port {} (PID: {})", port, process_info.pid);
            if let Err(e) = self.kill_process(process_info.pid).await {
                errors.push(format!("Port {} (PID {}): {}", port, process_info.pid, e));
            }
        }

        if !errors.is_empty() {
            let error_msg = errors.join("; ");
            return Err(anyhow::anyhow!("Some processes failed to kill: {}", error_msg));
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
}
