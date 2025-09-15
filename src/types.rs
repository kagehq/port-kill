use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessInfo {
    pub pid: i32,
    pub port: u16,
    pub command: String,
    pub name: String,
    pub container_id: Option<String>,
    pub container_name: Option<String>,
    pub command_line: Option<String>,
    pub working_directory: Option<String>,
    pub process_group: Option<String>,  // NEW: Group processes by type (e.g., "Node.js", "Python", "Docker")
    pub project_name: Option<String>,   // NEW: Extract project name from working directory
    pub cpu_usage: Option<f64>,         // NEW: CPU usage percentage
    pub memory_usage: Option<u64>,      // NEW: Memory usage in bytes
    pub memory_percentage: Option<f64>, // NEW: Memory usage percentage
}

#[derive(Debug, Clone)]
pub struct ProcessUpdate {
    pub processes: HashMap<u16, ProcessInfo>,
    pub count: usize,
}

impl ProcessUpdate {
    pub fn new(processes: HashMap<u16, ProcessInfo>) -> Self {
        let count = processes.len();
        Self { processes, count }
    }

    pub fn empty() -> Self {
        Self {
            processes: HashMap::new(),
            count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusBarInfo {
    pub text: String,
    pub tooltip: String,
}

impl StatusBarInfo {
    pub fn from_process_count(count: usize) -> Self {
        let text = count.to_string(); // Just show the number

        let tooltip = if count == 0 {
            "No development processes running".to_string()
        } else {
            format!("{} development process(es) running", count)
        };

        Self { text, tooltip }
    }
    
    pub fn from_processes_with_status(processes: &std::collections::HashMap<u16, ProcessInfo>) -> Self {
        let count = processes.len();
        
        if count == 0 {
            return Self {
                text: "0".to_string(),
                tooltip: "No development processes running".to_string(),
            };
        }
        
        // Analyze process status
        let mut high_cpu_count = 0;
        let mut high_memory_count = 0;
        let mut docker_count = 0;
        let mut groups: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        for process_info in processes.values() {
            // Check for high resource usage
            if let Some(cpu) = process_info.cpu_usage {
                if cpu > 50.0 {
                    high_cpu_count += 1;
                }
            }
            
            if let Some(memory) = process_info.memory_percentage {
                if memory > 10.0 {
                    high_memory_count += 1;
                }
            }
            
            // Count Docker containers
            if process_info.container_id.is_some() {
                docker_count += 1;
            }
            
            // Collect process groups
            if let Some(ref group) = process_info.process_group {
                groups.insert(group.clone());
            }
        }
        
        // Create status text with indicators
        let mut status_parts = vec![count.to_string()];
        
        if high_cpu_count > 0 {
            status_parts.push(format!("🔥{}", high_cpu_count));
        }
        
        if high_memory_count > 0 {
            status_parts.push(format!("💾{}", high_memory_count));
        }
        
        if docker_count > 0 {
            status_parts.push(format!("🐳{}", docker_count));
        }
        
        let text = status_parts.join(" ");
        
        // Create detailed tooltip
        let mut tooltip_parts = vec![format!("{} development process(es) running", count)];
        
        if !groups.is_empty() {
            tooltip_parts.push(format!("Groups: {}", groups.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
        }
        
        if high_cpu_count > 0 {
            tooltip_parts.push(format!("{} high CPU processes", high_cpu_count));
        }
        
        if high_memory_count > 0 {
            tooltip_parts.push(format!("{} high memory processes", high_memory_count));
        }
        
        if docker_count > 0 {
            tooltip_parts.push(format!("{} Docker containers", docker_count));
        }
        
        let tooltip = tooltip_parts.join(" | ");
        
        Self { text, tooltip }
    }
}

impl ProcessInfo {
    /// Determine the process group based on the command and name
    pub fn determine_process_group(&self) -> Option<String> {
        let name_lower = self.name.to_lowercase();
        let command_lower = self.command.to_lowercase();
        
        // Check for common development tools
        if name_lower.contains("node") || command_lower.contains("node") {
            Some("Node.js".to_string())
        } else if name_lower.contains("python") || command_lower.contains("python") {
            Some("Python".to_string())
        } else if name_lower.contains("java") || command_lower.contains("java") {
            Some("Java".to_string())
        } else if name_lower.contains("go") || command_lower.contains("go ") {
            Some("Go".to_string())
        } else if name_lower.contains("rust") || command_lower.contains("cargo") {
            Some("Rust".to_string())
        } else if name_lower.contains("php") || command_lower.contains("php") {
            Some("PHP".to_string())
        } else if name_lower.contains("ruby") || command_lower.contains("ruby") {
            Some("Ruby".to_string())
        } else if name_lower.contains("docker") || command_lower.contains("docker") {
            Some("Docker".to_string())
        } else if name_lower.contains("nginx") || command_lower.contains("apache") {
            Some("Web Server".to_string())
        } else if name_lower.contains("postgres") || name_lower.contains("mysql") || name_lower.contains("redis") {
            Some("Database".to_string())
        } else {
            None
        }
    }
    
    /// Extract project name from working directory
    pub fn extract_project_name(&self) -> Option<String> {
        if let Some(ref work_dir) = self.working_directory {
            // Get the last part of the path (project folder name)
            let path_parts: Vec<&str> = work_dir.split('/').collect();
            if let Some(last_part) = path_parts.last() {
                if !last_part.is_empty() && *last_part != "~" {
                    return Some(last_part.to_string());
                }
            }
            
            // Try to find a meaningful project name from the path
            for part in path_parts.iter().rev() {
                if !part.is_empty() && *part != "~" && *part != "home" && *part != "Users" {
                    // Check if this looks like a project directory
                    if part.contains("project") || part.contains("app") || part.contains("service") ||
                       part.contains("api") || part.contains("frontend") || part.contains("backend") ||
                       part.contains("client") || part.contains("server") {
                        return Some(part.to_string());
                    }
                }
            }
        }
        None
    }
    
    /// Get the full project path context
    pub fn get_project_context(&self) -> Option<String> {
        if let Some(ref work_dir) = self.working_directory {
            // Return the full working directory path
            Some(work_dir.clone())
        } else {
            None
        }
    }
    
    /// Get a human-readable project description
    pub fn get_project_description(&self) -> String {
        if let Some(ref project) = self.project_name {
            if let Some(ref context) = self.get_project_context() {
                format!("{} ({})", project, context)
            } else {
                project.clone()
            }
        } else if let Some(ref context) = self.get_project_context() {
            context.clone()
        } else {
            "Unknown Project".to_string()
        }
    }
    
    /// Get a more descriptive display name
    pub fn get_display_name(&self) -> String {
        // Try to create a more descriptive name
        let mut display_parts = Vec::new();
        
        // Add process name
        display_parts.push(self.name.clone());
        
        // Add project context if available
        if let Some(ref project) = self.project_name {
            display_parts.push(format!("[{}]", project));
        }
        
        // Add process group context
        if let Some(ref group) = self.process_group {
            display_parts.push(format!("({})", group));
        }
        
        // Add port context for clarity
        display_parts.push(format!(":{}", self.port));
        
        display_parts.join(" ")
    }
    
    /// Get a short, clean process name for status display
    pub fn get_short_name(&self) -> String {
        // Extract just the executable name without path
        let name = if self.name.contains('/') {
            self.name.split('/').last().unwrap_or(&self.name)
        } else if self.name.contains('\\') {
            self.name.split('\\').last().unwrap_or(&self.name)
        } else {
            &self.name
        };
        
        // Remove common extensions
        let name = name
            .trim_end_matches(".exe")
            .trim_end_matches(".dll")
            .trim_end_matches(".so");
        
        name.to_string()
    }
    
    /// Get a detailed process description
    pub fn get_detailed_description(&self) -> String {
        let mut parts = Vec::new();
        
        // Process name and port
        parts.push(format!("{} on port {}", self.get_short_name(), self.port));
        
        // Add command line if available and different from name
        if let Some(ref cmd_line) = self.command_line {
            if cmd_line != &self.name && !cmd_line.is_empty() {
                parts.push(format!("({})", cmd_line));
            }
        }
        
        // Add working directory if available
        if let Some(ref work_dir) = self.working_directory {
            parts.push(format!("in {}", work_dir));
        }
        
        // Add container info
        if let (Some(_container_id), Some(container_name)) = (&self.container_id, &self.container_name) {
            parts.push(format!("[Docker: {}]", container_name));
        }
        
        parts.join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHistoryEntry {
    pub pid: i32,
    pub port: u16,
    pub process_name: String,
    pub process_group: Option<String>,
    pub project_name: Option<String>,
    pub killed_at: DateTime<Utc>,
    pub killed_by: String, // "user", "bulk", "auto"
    pub command_line: Option<String>,
    pub working_directory: Option<String>,
}

impl ProcessHistoryEntry {
    pub fn new(process_info: &ProcessInfo, killed_by: String) -> Self {
        Self {
            pid: process_info.pid,
            port: process_info.port,
            process_name: process_info.name.clone(),
            process_group: process_info.process_group.clone(),
            project_name: process_info.project_name.clone(),
            killed_at: Utc::now(),
            killed_by,
            command_line: process_info.command_line.clone(),
            working_directory: process_info.working_directory.clone(),
        }
    }
    
    pub fn get_display_name(&self) -> String {
        if let Some(ref group) = self.process_group {
            if let Some(ref project) = self.project_name {
                format!("{} ({})", group, project)
            } else {
                group.clone()
            }
        } else if let Some(ref project) = self.project_name {
            format!("{} ({})", self.process_name, project)
        } else {
            self.process_name.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessHistory {
    entries: Vec<ProcessHistoryEntry>,
    max_entries: usize,
}

impl ProcessHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }
    
    pub fn add_entry(&mut self, entry: ProcessHistoryEntry) {
        self.entries.push(entry);
        
        // Keep only the most recent entries
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }
    
    pub fn get_recent_entries(&self, limit: usize) -> &[ProcessHistoryEntry] {
        let start = if self.entries.len() > limit {
            self.entries.len() - limit
        } else {
            0
        };
        &self.entries[start..]
    }
    
    pub fn get_entries_by_group(&self, group: &str) -> Vec<&ProcessHistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.process_group.as_ref().map_or(false, |g| g == group))
            .collect()
    }
    
    pub fn get_entries_by_project(&self, project: &str) -> Vec<&ProcessHistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.project_name.as_ref().map_or(false, |p| p == project))
            .collect()
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        fs::write(file_path, json)?;
        Ok(())
    }
    
    pub fn load_from_file(file_path: &str, max_entries: usize) -> Result<Self, Box<dyn std::error::Error>> {
        if Path::new(file_path).exists() {
            let json = fs::read_to_string(file_path)?;
            let entries: Vec<ProcessHistoryEntry> = serde_json::from_str(&json)?;
            Ok(Self {
                entries,
                max_entries,
            })
        } else {
            Ok(Self::new(max_entries))
        }
    }
    
    pub fn get_history_file_path() -> String {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/.port-kill-history.json", home_dir)
    }
}
