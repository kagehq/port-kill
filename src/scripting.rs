use crate::{
    process_monitor::ProcessMonitor,
    types::ProcessInfo,
    cli::Args,
    file_monitor::FileMonitor,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::fs;

/// Port guard configuration
#[derive(Debug, Clone)]
enum GuardConfig {
    KillAll,                    // Kill any process on this port
    AllowOnly(String),          // Only allow specific process name
}

/// Scripting engine for port-kill
pub struct ScriptEngine {
    process_monitor: Arc<Mutex<ProcessMonitor>>,
    file_monitor: FileMonitor,
    args: Args,
    port_handlers: HashMap<u16, Vec<Box<dyn Fn(ProcessInfo) + Send + Sync>>>,
    last_processes: HashMap<u16, ProcessInfo>, // Track last known processes to detect changes
    port_guards: HashMap<u16, GuardConfig>,    // Port guard configurations
    file_guards: HashMap<String, GuardConfig>, // File guard configurations
}

impl ScriptEngine {
    /// Create a new scripting engine
    pub fn new(process_monitor: Arc<Mutex<ProcessMonitor>>, args: Args) -> Self {
        Self {
            process_monitor,
            file_monitor: FileMonitor::new(),
            args,
            port_handlers: HashMap::new(),
            last_processes: HashMap::new(),
            port_guards: HashMap::new(),
            file_guards: HashMap::new(),
        }
    }

    /// Execute a script
    pub async fn execute(&mut self, script: &str) -> Result<()> {
        match self.args.script_lang.as_str() {
            "js" => self.execute_javascript(script).await,
            "python" => self.execute_python(script).await,
            _ => Err(anyhow::anyhow!("Unsupported scripting language: {}", self.args.script_lang)),
        }
    }

    /// Execute JavaScript script
    async fn execute_javascript(&mut self, script: &str) -> Result<()> {
        println!("🚀 Executing JavaScript script...");
        println!("📝 Script: {}", script);
        
        // For now, we'll implement a simple command parser
        // Later we'll integrate a proper JavaScript runtime
        self.parse_and_execute_commands(script).await
    }

    /// Execute Python script
    async fn execute_python(&mut self, script: &str) -> Result<()> {
        println!("🚀 Executing Python script...");
        println!("📝 Script: {}", script);
        
        // For now, we'll implement a simple command parser
        // Later we'll integrate a proper Python runtime
        self.parse_and_execute_commands(script).await
    }

    /// Parse and execute simple commands (temporary implementation)
    async fn parse_and_execute_commands(&mut self, script: &str) -> Result<()> {
        let lines: Vec<&str> = script.lines().collect();
        
        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with("#") {
                continue; // Skip empty lines and comments
            }

            // Handle multiple commands on one line (separated by semicolons)
            let commands: Vec<&str> = line.split(';').map(|s| s.trim()).collect();
            
            for command in commands {
                if command.is_empty() {
                    continue;
                }
                
                if command.starts_with("onPort(") {
                    self.parse_on_port_command(command).await?;
                } else if command.starts_with("kill(") {
                    self.parse_kill_command(command).await?;
                } else if command.starts_with("listPorts(") {
                    self.parse_list_ports_command(command).await?;
                } else if command.starts_with("killPort(") {
                    self.parse_kill_port_command(command).await?;
                } else if command.starts_with("getProcess(") {
                    self.parse_get_process_command(command).await?;
                } else if command.starts_with("log(") {
                    self.parse_log_command(command).await?;
                } else if command.starts_with("wait(") {
                    self.parse_wait_command(command).await?;
                } else if command.starts_with("guardPort(") {
                    self.parse_guard_port_command(command).await?;
                } else if command.starts_with("killFile(") {
                    self.parse_kill_file_command(command).await?;
                } else if command.starts_with("guardFile(") {
                    self.parse_guard_file_command(command).await?;
                } else if command.starts_with("killFileExt(") {
                    self.parse_kill_file_ext_command(command).await?;
                } else if command.starts_with("listFileProcesses(") {
                    self.parse_list_file_processes_command(command).await?;
                } else {
                    println!("⚠️  Unknown command: {}", command);
                }
            }
        }

        // Start monitoring if we have any port handlers or guards
        if !self.port_handlers.is_empty() || !self.port_guards.is_empty() {
            println!("📡 Starting port monitoring for script...");
            self.start_monitoring().await?;
        }

        Ok(())
    }

    /// Parse onPort command
    async fn parse_on_port_command(&mut self, line: &str) -> Result<()> {
        // Simple parsing: onPort(3000, callback)
        // For now, we'll just register a basic handler
        if let Some(port_str) = self.extract_port_from_onport(line) {
            if let Ok(port) = port_str.parse::<u16>() {
                println!("📌 Registered handler for port {}", port);
                
                // Register a simple handler that logs when processes are detected
                let handler = Box::new(move |process: ProcessInfo| {
                    println!("🔍 Process detected on port {}: {} (PID: {})", 
                             process.port, process.name, process.pid);
                });
                
                self.port_handlers.entry(port).or_insert_with(Vec::new).push(handler);
            }
        }
        Ok(())
    }

    /// Parse kill command
    async fn parse_kill_command(&mut self, line: &str) -> Result<()> {
        // Simple parsing: kill(pid) or kill(port)
        if let Some(pid_str) = self.extract_pid_from_kill(line) {
            if let Ok(pid) = pid_str.parse::<u32>() {
                println!("🔪 Killing process with PID: {}", pid);
                // TODO: Implement actual kill functionality
            }
        }
        Ok(())
    }

    /// Parse listPorts command
    async fn parse_list_ports_command(&mut self, _line: &str) -> Result<()> {
        println!("📋 Listing monitored ports...");
        let ports = self.args.get_ports_to_monitor();
        for port in ports {
            println!("  • Port {}", port);
        }
        Ok(())
    }

    /// Parse killPort command
    async fn parse_kill_port_command(&mut self, line: &str) -> Result<()> {
        if let Some(port_str) = self.extract_port_from_killport(line) {
            if let Ok(port) = port_str.parse::<u16>() {
                println!("🔪 Killing all processes on port {}", port);
                // TODO: Implement actual port killing functionality
                // For now, we'll use the existing kill_all_processes with specific port
                let ports_to_kill = vec![port];
                if let Err(e) = crate::process_monitor::kill_all_processes(&ports_to_kill, &self.args) {
                    println!("❌ Failed to kill processes on port {}: {}", port, e);
                } else {
                    println!("✅ Successfully killed processes on port {}", port);
                }
            }
        }
        Ok(())
    }

    /// Parse getProcess command
    async fn parse_get_process_command(&mut self, line: &str) -> Result<()> {
        if let Some(port_str) = self.extract_port_from_getprocess(line) {
            if let Ok(port) = port_str.parse::<u16>() {
                println!("🔍 Getting process info for port {}", port);
                // TODO: Implement process info retrieval
                println!("  Port {}: Process information would be displayed here", port);
            }
        }
        Ok(())
    }

    /// Parse log command
    async fn parse_log_command(&mut self, line: &str) -> Result<()> {
        if let Some(message) = self.extract_message_from_log(line) {
            println!("📝 LOG: {}", message);
        }
        Ok(())
    }

    /// Parse wait command
    async fn parse_wait_command(&mut self, line: &str) -> Result<()> {
        if let Some(seconds_str) = self.extract_seconds_from_wait(line) {
            if let Ok(seconds) = seconds_str.parse::<u64>() {
                println!("⏳ Waiting {} seconds...", seconds);
                tokio::time::sleep(tokio::time::Duration::from_secs(seconds)).await;
                println!("✅ Wait completed");
            }
        }
        Ok(())
    }

    /// Parse guardPort command
    async fn parse_guard_port_command(&mut self, line: &str) -> Result<()> {
        if let Some((port_str, allowed_name)) = self.extract_guard_port_params(line) {
            if let Ok(port) = port_str.parse::<u16>() {
                if let Some(name) = allowed_name {
                    println!("🛡️  Guarding port {} - only allowing '{}'", port, name);
                    self.port_guards.insert(port, GuardConfig::AllowOnly(name.to_string()));
                } else {
                    println!("🛡️  Guarding port {} - killing all processes", port);
                    self.port_guards.insert(port, GuardConfig::KillAll);
                }
            }
        }
        Ok(())
    }


    /// Parse killFile command
    async fn parse_kill_file_command(&mut self, line: &str) -> Result<()> {
        if let Some(file_path) = self.extract_file_path_from_killfile(line) {
            println!("🔪 Killing all processes with file '{}' open", file_path);
            if let Ok(processes) = self.file_monitor.find_processes_with_file(&file_path) {
                for process in processes {
                    println!("  Killing process: {} (PID: {})", process.name, process.pid);
                    if let Err(e) = self.process_monitor.lock().await.kill_process(process.pid).await {
                        println!("❌ Failed to kill process {}: {}", process.pid, e);
                    } else {
                        println!("✅ Successfully killed process {}", process.pid);
                    }
                }
            } else {
                println!("⚠️  No processes found with file '{}' open", file_path);
            }
        }
        Ok(())
    }

    /// Parse guardFile command
    async fn parse_guard_file_command(&mut self, line: &str) -> Result<()> {
        if let Some((file_path, allowed_name)) = self.extract_guard_file_params(line) {
            if let Some(name) = allowed_name {
                println!("🛡️  Guarding file '{}' - only allowing '{}'", file_path, name);
                self.file_guards.insert(file_path.to_string(), GuardConfig::AllowOnly(name.to_string()));
            } else {
                println!("🛡️  Guarding file '{}' - killing all processes that open it", file_path);
                self.file_guards.insert(file_path.to_string(), GuardConfig::KillAll);
            }
        }
        Ok(())
    }

    /// Parse killFileExt command
    async fn parse_kill_file_ext_command(&mut self, line: &str) -> Result<()> {
        if let Some(extension) = self.extract_extension_from_killfileext(line) {
            println!("🔪 Killing all processes with '{}' files open", extension);
            if let Ok(processes) = self.file_monitor.find_processes_with_extension(&extension) {
                for process in processes {
                    println!("  Killing process: {} (PID: {})", process.name, process.pid);
                    if let Err(e) = self.process_monitor.lock().await.kill_process(process.pid).await {
                        println!("❌ Failed to kill process {}: {}", process.pid, e);
                    } else {
                        println!("✅ Successfully killed process {}", process.pid);
                    }
                }
            } else {
                println!("⚠️  No processes found with '{}' files open", extension);
            }
        }
        Ok(())
    }

    /// Parse listFileProcesses command
    async fn parse_list_file_processes_command(&mut self, line: &str) -> Result<()> {
        if let Some(file_path) = self.extract_file_path_from_listfileprocesses(line) {
            println!("📋 Listing processes with file '{}' open:", file_path);
            if let Ok(processes) = self.file_monitor.find_processes_with_file(&file_path) {
                if processes.is_empty() {
                    println!("  No processes found with file '{}' open", file_path);
                } else {
                    for process in processes {
                        println!("  • {} (PID: {})", process.name, process.pid);
                    }
                }
            } else {
                println!("❌ Failed to check processes for file '{}'", file_path);
            }
        }
        Ok(())
    }

    /// Extract port number from onPort command
    fn extract_port_from_onport<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple regex-like parsing: onPort(3000, ...)
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(',') {
                return Some(&line[start+1..start+1+end]);
            }
        }
        None
    }

    /// Extract PID from kill command
    fn extract_pid_from_kill<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple parsing: kill(1234)
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                return Some(&line[start+1..start+1+end]);
            }
        }
        None
    }

    /// Extract port from killPort command
    fn extract_port_from_killport<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple parsing: killPort(3000)
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                return Some(&line[start+1..start+1+end]);
            }
        }
        None
    }

    /// Extract port from getProcess command
    fn extract_port_from_getprocess<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple parsing: getProcess(3000)
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                return Some(&line[start+1..start+1+end]);
            }
        }
        None
    }

    /// Extract message from log command
    fn extract_message_from_log<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple parsing: log("message")
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let content = &line[start+1..start+1+end];
                // Remove quotes if present
                if content.starts_with('"') && content.ends_with('"') {
                    return Some(&content[1..content.len()-1]);
                }
                return Some(content);
            }
        }
        None
    }

    /// Extract seconds from wait command
    fn extract_seconds_from_wait<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple parsing: wait(5)
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                return Some(&line[start+1..start+1+end]);
            }
        }
        None
    }

    /// Extract guardPort parameters
    fn extract_guard_port_params<'a>(&self, line: &'a str) -> Option<(&'a str, Option<&'a str>)> {
        // Simple parsing: guardPort(3000) or guardPort(3000, "my-dev-server")
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let content = &line[start+1..start+1+end];
                if let Some(comma_pos) = content.find(',') {
                    let port_str = content[..comma_pos].trim();
                    let name_str = content[comma_pos+1..].trim();
                    // Remove quotes if present
                    let name = if name_str.starts_with('"') && name_str.ends_with('"') {
                        Some(&name_str[1..name_str.len()-1])
                    } else {
                        Some(name_str)
                    };
                    return Some((port_str, name));
                } else {
                    return Some((content, None));
                }
            }
        }
        None
    }


    /// Extract file path from killFile command
    fn extract_file_path_from_killfile<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple parsing: killFile("filename.ext")
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let content = &line[start+1..start+1+end];
                // Remove quotes if present
                if content.starts_with('"') && content.ends_with('"') {
                    return Some(&content[1..content.len()-1]);
                }
                return Some(content);
            }
        }
        None
    }

    /// Extract guardFile parameters
    fn extract_guard_file_params<'a>(&self, line: &'a str) -> Option<(&'a str, Option<&'a str>)> {
        // Simple parsing: guardFile("filename.ext") or guardFile("filename.ext", "allowed-process")
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let content = &line[start+1..start+1+end];
                if let Some(comma_pos) = content.find(',') {
                    let file_path = content[..comma_pos].trim();
                    let name_str = content[comma_pos+1..].trim();
                    // Remove quotes if present
                    let name = if name_str.starts_with('"') && name_str.ends_with('"') {
                        Some(&name_str[1..name_str.len()-1])
                    } else {
                        Some(name_str)
                    };
                    // Remove quotes from file path if present
                    let file_path = if file_path.starts_with('"') && file_path.ends_with('"') {
                        &file_path[1..file_path.len()-1]
                    } else {
                        file_path
                    };
                    return Some((file_path, name));
                } else {
                    // Remove quotes from file path if present
                    let file_path = if content.starts_with('"') && content.ends_with('"') {
                        &content[1..content.len()-1]
                    } else {
                        content
                    };
                    return Some((file_path, None));
                }
            }
        }
        None
    }

    /// Extract extension from killFileExt command
    fn extract_extension_from_killfileext<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple parsing: killFileExt(".lock")
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let content = &line[start+1..start+1+end];
                // Remove quotes if present
                if content.starts_with('"') && content.ends_with('"') {
                    return Some(&content[1..content.len()-1]);
                }
                return Some(content);
            }
        }
        None
    }

    /// Extract file path from listFileProcesses command
    fn extract_file_path_from_listfileprocesses<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Simple parsing: listFileProcesses("filename.ext")
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let content = &line[start+1..start+1+end];
                // Remove quotes if present
                if content.starts_with('"') && content.ends_with('"') {
                    return Some(&content[1..content.len()-1]);
                }
                return Some(content);
            }
        }
        None
    }

    /// Start monitoring for script handlers
    async fn start_monitoring(&mut self) -> Result<()> {
        println!("🔄 Starting continuous monitoring...");
        println!("💡 Press Ctrl+C to stop");
        
        // Start the monitoring loop
        let monitor = self.process_monitor.clone();
        let watched_ports: Vec<u16> = self.port_handlers.keys().cloned().collect();
        let guard_ports: Vec<u16> = self.port_guards.keys().cloned().collect();
        let all_monitored_ports: Vec<u16> = watched_ports.iter().chain(guard_ports.iter()).cloned().collect();
        let port_guards = self.port_guards.clone();
        
        tokio::spawn(async move {
            let mut last_processes: HashMap<u16, ProcessInfo> = HashMap::new();
            
            loop {
                if let Ok(mut monitor) = monitor.try_lock() {
                    if let Ok(processes) = monitor.scan_processes().await {
                        // Get current ports first
                        let current_ports: std::collections::HashSet<u16> = processes.keys().cloned().collect();
                        
                        for (port, process_info) in processes {
                            if all_monitored_ports.contains(&port) {
                                // Check if this is a new or changed process
                                let is_new = !last_processes.contains_key(&port);
                                let is_changed = if let Some(last_process) = last_processes.get(&port) {
                                    last_process.pid != process_info.pid || last_process.name != process_info.name
                                } else {
                                    false
                                };
                                
                                if is_new {
                                    println!("🟢 NEW: Process started on port {}: {} (PID: {})", 
                                             process_info.port, process_info.name, process_info.pid);
                                    
                                    // Check if this port has a guard and handle accordingly
                                    if let Some(guard_config) = port_guards.get(&port) {
                                        match guard_config {
                                            GuardConfig::KillAll => {
                                                println!("🚨 Unauthorized process on port {}: {} (PID: {}) - KILLING", 
                                                         port, process_info.name, process_info.pid);
                                                if let Err(e) = monitor.kill_process(process_info.pid).await {
                                                    println!("❌ Failed to kill process {}: {}", process_info.pid, e);
                                                } else {
                                                    println!("✅ Successfully killed unauthorized process {} on port {}", 
                                                             process_info.pid, port);
                                                }
                                            }
                                            GuardConfig::AllowOnly(allowed_name) => {
                                                if process_info.name != *allowed_name {
                                                    println!("🚨 Unauthorized process '{}' on port {}: {} (PID: {}) - KILLING", 
                                                             process_info.name, port, process_info.name, process_info.pid);
                                                    if let Err(e) = monitor.kill_process(process_info.pid).await {
                                                        println!("❌ Failed to kill process {}: {}", process_info.pid, e);
                                                    } else {
                                                        println!("✅ Successfully killed unauthorized process {} on port {}", 
                                                                 process_info.pid, port);
                                                    }
                                                } else {
                                                    println!("✅ Authorized process '{}' (PID: {}) on port {}", 
                                                             process_info.name, process_info.pid, port);
                                                }
                                            }
                                        }
                                    }
                                } else if is_changed {
                                    println!("🔄 CHANGED: Process on port {}: {} (PID: {})", 
                                             process_info.port, process_info.name, process_info.pid);
                                }
                                
                                // Update our tracking
                                last_processes.insert(port, process_info);
                            }
                        }
                        
                        // Check for processes that disappeared
                        for (port, last_process) in last_processes.iter() {
                            if all_monitored_ports.contains(port) && !current_ports.contains(port) {
                                println!("🔴 REMOVED: Process stopped on port {}: {} (PID: {})", 
                                         port, last_process.name, last_process.pid);
                            }
                        }
                        
                        // Clean up tracking for ports that are no longer active
                        last_processes.retain(|port, _| current_ports.contains(port));
                    }
                }
                
                // Sleep for 2 seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });

        // Keep the main thread alive
        tokio::signal::ctrl_c().await?;
        println!("🛑 Script monitoring stopped");
        
        Ok(())
    }
}

/// Load script from file
pub fn load_script_file(file_path: &str) -> Result<String> {
    fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read script file '{}': {}", file_path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_port_from_onport() {
        let engine = ScriptEngine::new(
            Arc::new(Mutex::new(ProcessMonitor::new(
                crossbeam_channel::bounded(100).0,
                vec![3000],
                false,
                false,
                None,
                false,
            ).unwrap())),
            Args {
                script_lang: "js".to_string(),
                ..Default::default()
            }
        );
        
        assert_eq!(engine.extract_port_from_onport("onPort(3000, callback)"), Some("3000"));
        assert_eq!(engine.extract_port_from_onport("onPort(8080, proc => kill(proc.pid))"), Some("8080"));
    }

    #[test]
    fn test_extract_pid_from_kill() {
        let engine = ScriptEngine::new(
            Arc::new(Mutex::new(ProcessMonitor::new(
                crossbeam_channel::bounded(100).0,
                vec![3000],
                false,
                false,
                None,
                false,
            ).unwrap())),
            Args {
                script_lang: "js".to_string(),
                ..Default::default()
            }
        );
        
        assert_eq!(engine.extract_pid_from_kill("kill(1234)"), Some("1234"));
        assert_eq!(engine.extract_pid_from_kill("kill(5678)"), Some("5678"));
        assert_eq!(engine.extract_port_from_killport("killPort(3000)"), Some("3000"));
        assert_eq!(engine.extract_port_from_getprocess("getProcess(8080)"), Some("8080"));
        assert_eq!(engine.extract_message_from_log("log(\"Hello World\")"), Some("Hello World"));
        assert_eq!(engine.extract_seconds_from_wait("wait(5)"), Some("5"));
    }
}
