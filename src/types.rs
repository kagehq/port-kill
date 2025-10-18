use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
    pub process_group: Option<String>, // NEW: Group processes by type (e.g., "Node.js", "Python", "Docker")
    pub project_name: Option<String>,  // NEW: Extract project name from working directory
    pub cpu_usage: Option<f64>,        // NEW: CPU usage percentage
    pub memory_usage: Option<u64>,     // NEW: Memory usage in bytes
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

    pub fn from_processes_with_status(
        processes: &std::collections::HashMap<u16, ProcessInfo>,
    ) -> Self {
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
            tooltip_parts.push(format!(
                "Groups: {}",
                groups
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
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
        } else if name_lower.contains("postgres")
            || name_lower.contains("mysql")
            || name_lower.contains("redis")
        {
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
                    if part.contains("project")
                        || part.contains("app")
                        || part.contains("service")
                        || part.contains("api")
                        || part.contains("frontend")
                        || part.contains("backend")
                        || part.contains("client")
                        || part.contains("server")
                    {
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
        if let (Some(_container_id), Some(container_name)) =
            (&self.container_id, &self.container_name)
        {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequentOffender {
    pub process_name: String,
    pub port: u16,
    pub kill_count: usize,
    pub first_killed: DateTime<Utc>,
    pub last_killed: DateTime<Utc>,
    pub process_group: Option<String>,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePatterns {
    pub total_kills: usize,
    pub peak_hour: Option<u32>,
    pub peak_day: Option<chrono::Weekday>,
    pub hour_distribution: std::collections::HashMap<u32, usize>,
    pub day_distribution: std::collections::HashMap<chrono::Weekday, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreSuggestions {
    pub suggested_ports: Vec<u16>,
    pub suggested_processes: Vec<String>,
    pub suggested_groups: Vec<String>,
    pub frequent_offenders: Vec<FrequentOffender>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatistics {
    pub total_kills: usize,
    pub unique_processes: usize,
    pub unique_ports: usize,
    pub unique_projects: usize,
    pub most_killed_process: Option<(String, usize)>,
    pub most_killed_port: Option<(u16, usize)>,
    pub most_killed_project: Option<(String, usize)>,
    pub top_processes: Vec<(String, usize)>,
    pub top_ports: Vec<(u16, usize)>,
    pub top_projects: Vec<(String, usize)>,
    pub average_kills_per_day: f64,
    pub oldest_kill: Option<DateTime<Utc>>,
    pub newest_kill: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConflict {
    pub port: u16,
    pub conflicting_processes: Vec<String>,
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    PortCollision,
    ResourceContention,
    AutoRestart,
    ParentChild,
    DevelopmentStack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub affected_processes: Vec<String>,
    pub frequency: String,
    pub recommendation: String,
    pub confidence: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    HotReload,
    AutoRestart,
    DevelopmentStack,
    ResourceIntensive,
    TimeBased,
    ProjectRelated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRecommendation {
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub action: String,
    pub impact: String,
    pub priority: RecommendationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    ProcessManagement,
    PortOptimization,
    ResourceOptimization,
    WorkflowImprovement,
    IgnoreList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub conflicts: Vec<ProcessConflict>,
    pub patterns: Vec<WorkflowPattern>,
    pub recommendations: Vec<SmartRecommendation>,
    pub summary: String,
    pub analysis_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortReservation {
    pub port: u16,
    pub project_name: String,
    pub process_name: String,
    pub reserved_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConflict {
    pub port: u16,
    pub existing_process: ProcessInfo,
    pub new_process: ProcessInfo,
    pub conflict_type: PortConflictType,
    pub resolution: Option<PortResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortConflictType {
    PortInUse,
    ProcessCollision,
    ResourceContention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortResolution {
    KillExisting,
    ReassignPort,
    BlockNewProcess,
    NotifyUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardStatus {
    pub is_active: bool,
    pub watched_ports: Vec<u16>,
    pub active_reservations: Vec<PortReservation>,
    pub conflicts_resolved: usize,
    pub last_activity: Option<DateTime<Utc>>,
    pub auto_resolve_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditResult {
    pub audit_timestamp: DateTime<Utc>,
    pub total_ports_scanned: usize,
    pub suspicious_processes: Vec<SuspiciousProcess>,
    pub approved_processes: Vec<ApprovedProcess>,
    pub security_score: f64, // 0.0 to 100.0
    pub recommendations: Vec<SecurityRecommendation>,
    pub baseline_comparison: Option<BaselineComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousProcess {
    pub port: u16,
    pub process_info: ProcessInfo,
    pub suspicion_reason: SuspicionReason,
    pub risk_level: RiskLevel,
    pub binary_hash: Option<String>,
    pub parent_process: Option<String>,
    pub network_interface: String,
    pub first_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspicionReason {
    SuspiciousPort,
    UnknownBinary,
    UnexpectedLocation,
    HighPrivilege,
    NetworkExposure,
    ProcessAnomaly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedProcess {
    pub port: u16,
    pub process_info: ProcessInfo,
    pub service_type: ServiceType,
    pub expected_location: String,
    pub binary_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    WebServer,
    Database,
    SSH,
    Mail,
    DNS,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRecommendation {
    pub title: String,
    pub description: String,
    pub action: String,
    pub priority: RiskLevel,
    pub affected_processes: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub baseline_file: String,
    pub new_processes: Vec<ProcessInfo>,
    pub removed_processes: Vec<ProcessInfo>,
    pub changed_processes: Vec<ProcessChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessChange {
    pub port: u16,
    pub old_process: ProcessInfo,
    pub new_process: ProcessInfo,
    pub change_type: ProcessChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessChangeType {
    BinaryChanged,
    LocationChanged,
    ArgumentsChanged,
    UserChanged,
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

    pub fn load_from_file(
        file_path: &str,
        max_entries: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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

    /// Get frequent offenders - processes that have been killed multiple times
    pub fn get_frequent_offenders(&self, min_kills: usize) -> Vec<FrequentOffender> {
        use std::collections::HashMap;

        let mut process_counts: HashMap<String, Vec<&ProcessHistoryEntry>> = HashMap::new();

        // Group entries by process name and port
        for entry in &self.entries {
            let key = format!("{}:{}", entry.process_name, entry.port);
            process_counts
                .entry(key)
                .or_insert_with(Vec::new)
                .push(entry);
        }

        // Find processes that have been killed multiple times
        let mut offenders = Vec::new();
        for (_key, entries) in process_counts {
            if entries.len() >= min_kills {
                let first_entry = entries[0];
                let last_killed = entries.iter().map(|e| e.killed_at).max().unwrap();
                let first_killed = entries.iter().map(|e| e.killed_at).min().unwrap();

                offenders.push(FrequentOffender {
                    process_name: first_entry.process_name.clone(),
                    port: first_entry.port,
                    kill_count: entries.len(),
                    first_killed,
                    last_killed,
                    process_group: first_entry.process_group.clone(),
                    project_name: first_entry.project_name.clone(),
                });
            }
        }

        // Sort by kill count (most frequent first)
        offenders.sort_by(|a, b| b.kill_count.cmp(&a.kill_count));
        offenders
    }

    /// Get time-based patterns - when processes are most commonly killed
    pub fn get_time_patterns(&self) -> TimePatterns {
        use std::collections::HashMap;

        let mut hour_counts: HashMap<u32, usize> = HashMap::new();
        let mut day_counts: HashMap<chrono::Weekday, usize> = HashMap::new();

        for entry in &self.entries {
            let hour = entry.killed_at.hour();
            *hour_counts.entry(hour).or_insert(0) += 1;

            let weekday = entry.killed_at.weekday();
            *day_counts.entry(weekday).or_insert(0) += 1;
        }

        // Find peak hours and days
        let peak_hour = hour_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| *hour);

        let peak_day = day_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(day, _)| *day);

        TimePatterns {
            total_kills: self.entries.len(),
            peak_hour,
            peak_day,
            hour_distribution: hour_counts,
            day_distribution: day_counts,
        }
    }

    /// Get auto-suggestions for ignore lists based on history
    pub fn get_ignore_suggestions(&self, min_kills: usize) -> IgnoreSuggestions {
        let frequent_offenders = self.get_frequent_offenders(min_kills);

        let mut suggested_ports = Vec::new();
        let mut suggested_processes = Vec::new();
        let mut suggested_groups = Vec::new();

        for offender in &frequent_offenders {
            // Suggest ports that are frequently killed (lowered threshold)
            if offender.kill_count >= min_kills {
                suggested_ports.push(offender.port);
            }

            // Suggest process names that are frequently killed
            if offender.kill_count >= min_kills {
                suggested_processes.push(offender.process_name.clone());
            }

            // Suggest groups that are frequently killed
            if let Some(ref group) = offender.process_group {
                if offender.kill_count >= min_kills {
                    suggested_groups.push(group.clone());
                }
            }
        }

        // Remove duplicates
        suggested_ports.sort();
        suggested_ports.dedup();
        suggested_processes.sort();
        suggested_processes.dedup();
        suggested_groups.sort();
        suggested_groups.dedup();

        IgnoreSuggestions {
            suggested_ports,
            suggested_processes,
            suggested_groups,
            frequent_offenders,
        }
    }

    /// Get statistics about the history
    pub fn get_statistics(&self) -> HistoryStatistics {
        if self.entries.is_empty() {
            return HistoryStatistics {
                total_kills: 0,
                unique_processes: 0,
                unique_ports: 0,
                unique_projects: 0,
                most_killed_process: None,
                most_killed_port: None,
                most_killed_project: None,
                top_processes: Vec::new(),
                top_ports: Vec::new(),
                top_projects: Vec::new(),
                average_kills_per_day: 0.0,
                oldest_kill: None,
                newest_kill: None,
            };
        }

        use std::collections::HashMap;

        let mut process_counts: HashMap<String, usize> = HashMap::new();
        let mut port_counts: HashMap<u16, usize> = HashMap::new();
        let mut project_counts: HashMap<String, usize> = HashMap::new();

        let mut oldest_kill = self.entries[0].killed_at;
        let mut newest_kill = self.entries[0].killed_at;

        for entry in &self.entries {
            // Use process_group if available, otherwise fall back to process_name
            let process_key = if let Some(ref group) = entry.process_group {
                group.clone()
            } else {
                entry.process_name.clone()
            };
            *process_counts.entry(process_key).or_insert(0) += 1;
            *port_counts.entry(entry.port).or_insert(0) += 1;

            if let Some(ref project) = entry.project_name {
                *project_counts.entry(project.clone()).or_insert(0) += 1;
            }

            if entry.killed_at < oldest_kill {
                oldest_kill = entry.killed_at;
            }
            if entry.killed_at > newest_kill {
                newest_kill = entry.killed_at;
            }
        }

        let most_killed_process = process_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, count)| (name.clone(), *count));

        let most_killed_port = port_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(port, count)| (*port, *count));

        let most_killed_project = project_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, count)| (name.clone(), *count));

        // Get top 3 processes, ports, and projects
        let total_unique_processes = process_counts.len();
        let total_unique_ports = port_counts.len();
        let total_unique_projects = project_counts.len();

        let mut top_processes: Vec<(String, usize)> = process_counts.into_iter().collect();
        top_processes.sort_by(|a, b| b.1.cmp(&a.1));
        top_processes.truncate(3);

        let mut top_ports: Vec<(u16, usize)> = port_counts.into_iter().collect();
        top_ports.sort_by(|a, b| b.1.cmp(&a.1));
        top_ports.truncate(3);

        let mut top_projects: Vec<(String, usize)> = project_counts.into_iter().collect();
        top_projects.sort_by(|a, b| b.1.cmp(&a.1));
        top_projects.truncate(3);

        // Calculate average kills per day
        let days_span = if oldest_kill != newest_kill {
            (newest_kill - oldest_kill).num_days() as f64
        } else {
            1.0
        };
        let average_kills_per_day = self.entries.len() as f64 / days_span.max(1.0);

        HistoryStatistics {
            total_kills: self.entries.len(),
            unique_processes: total_unique_processes,
            unique_ports: total_unique_ports,
            unique_projects: total_unique_projects,
            most_killed_process,
            most_killed_port,
            most_killed_project,
            top_processes,
            top_ports,
            top_projects,
            average_kills_per_day,
            oldest_kill: Some(oldest_kill),
            newest_kill: Some(newest_kill),
        }
    }

    /// Perform smart root cause analysis on the process history
    pub fn get_root_cause_analysis(&self) -> RootCauseAnalysis {
        let mut conflicts = Vec::new();
        let mut patterns = Vec::new();
        let mut recommendations = Vec::new();

        // Analyze conflicts
        conflicts.extend(self.analyze_port_conflicts());
        conflicts.extend(self.analyze_auto_restart_patterns());

        // Analyze workflow patterns
        patterns.extend(self.analyze_development_patterns());
        patterns.extend(self.analyze_time_patterns());

        // Generate smart recommendations
        recommendations.extend(self.generate_process_management_recommendations());
        recommendations.extend(self.generate_port_optimization_recommendations());
        recommendations.extend(self.generate_workflow_recommendations());

        // Generate summary
        let summary = self.generate_analysis_summary(&conflicts, &patterns, &recommendations);

        RootCauseAnalysis {
            conflicts,
            patterns,
            recommendations,
            summary,
            analysis_timestamp: Utc::now(),
        }
    }

    /// Analyze port conflicts and collisions
    fn analyze_port_conflicts(&self) -> Vec<ProcessConflict> {
        let mut conflicts = Vec::new();
        use std::collections::HashMap;

        // Group processes by port
        let mut port_processes: HashMap<u16, Vec<&ProcessHistoryEntry>> = HashMap::new();
        for entry in &self.entries {
            port_processes
                .entry(entry.port)
                .or_insert_with(Vec::new)
                .push(entry);
        }

        // Find ports with multiple different processes
        for (port, entries) in port_processes {
            if entries.len() > 1 {
                let unique_processes: std::collections::HashSet<String> =
                    entries.iter().map(|e| e.process_name.clone()).collect();

                if unique_processes.len() > 1 {
                    let conflicting_processes: Vec<String> = unique_processes.into_iter().collect();
                    let severity = if entries.len() > 5 {
                        ConflictSeverity::High
                    } else if entries.len() > 3 {
                        ConflictSeverity::Medium
                    } else {
                        ConflictSeverity::Low
                    };

                    conflicts.push(ProcessConflict {
                        port,
                        conflicting_processes,
                        conflict_type: ConflictType::PortCollision,
                        severity,
                        recommendation: format!("Consider using different ports for different services. Port {} is being used by multiple processes.", port),
                    });
                }
            }
        }

        conflicts
    }

    /// Analyze auto-restart patterns
    fn analyze_auto_restart_patterns(&self) -> Vec<ProcessConflict> {
        let mut conflicts = Vec::new();
        use std::collections::HashMap;

        // Group by process name and port
        let mut process_groups: HashMap<String, Vec<&ProcessHistoryEntry>> = HashMap::new();
        for entry in &self.entries {
            let key = format!("{}:{}", entry.process_name, entry.port);
            process_groups
                .entry(key)
                .or_insert_with(Vec::new)
                .push(entry);
        }

        // Find processes that restart frequently
        for (key, entries) in process_groups {
            if entries.len() >= 3 {
                // Check if kills are close in time (indicating auto-restart)
                let mut sorted_entries = entries.clone();
                sorted_entries.sort_by(|a, b| a.killed_at.cmp(&b.killed_at));

                let mut short_intervals = 0;
                for i in 1..sorted_entries.len() {
                    let interval = sorted_entries[i].killed_at - sorted_entries[i - 1].killed_at;
                    if interval.num_minutes() < 5 {
                        short_intervals += 1;
                    }
                }

                if short_intervals > 0 {
                    let parts: Vec<&str> = key.split(':').collect();
                    let process_name = parts[0].to_string();
                    let port: u16 = parts[1].parse().unwrap_or(0);

                    conflicts.push(ProcessConflict {
                        port,
                        conflicting_processes: vec![process_name.clone()],
                        conflict_type: ConflictType::AutoRestart,
                        severity: if short_intervals > 3 { ConflictSeverity::High } else { ConflictSeverity::Medium },
                        recommendation: format!("Process '{}' appears to auto-restart. Killing it may not be effective. Consider adding to ignore list or investigating the root cause.", process_name),
                    });
                }
            }
        }

        conflicts
    }

    /// Analyze development workflow patterns
    fn analyze_development_patterns(&self) -> Vec<WorkflowPattern> {
        let mut patterns = Vec::new();

        // Look for hot reload patterns (same process killed multiple times in short intervals)
        let mut process_groups: std::collections::HashMap<String, Vec<&ProcessHistoryEntry>> =
            std::collections::HashMap::new();
        for entry in &self.entries {
            process_groups
                .entry(entry.process_name.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }

        for (process_name, entries) in process_groups {
            if entries.len() >= 3 {
                let mut sorted_entries = entries.clone();
                sorted_entries.sort_by(|a, b| a.killed_at.cmp(&b.killed_at));

                // Check for hot reload pattern (kills within 1-2 minutes)
                let mut hot_reload_count = 0;
                for i in 1..sorted_entries.len() {
                    let interval = sorted_entries[i].killed_at - sorted_entries[i - 1].killed_at;
                    if interval.num_minutes() <= 2 {
                        hot_reload_count += 1;
                    }
                }

                if hot_reload_count >= 2 {
                    patterns.push(WorkflowPattern {
                        pattern_type: PatternType::HotReload,
                        description: format!("Process '{}' shows hot reload behavior", process_name),
                        affected_processes: vec![process_name.clone()],
                        frequency: format!("{} times in short intervals", hot_reload_count),
                        recommendation: "This appears to be a development server with hot reload. Consider adding to ignore list during development.".to_string(),
                        confidence: 0.8,
                    });
                }
            }
        }

        patterns
    }

    /// Analyze time-based patterns
    fn analyze_time_patterns(&self) -> Vec<WorkflowPattern> {
        let mut patterns = Vec::new();

        if self.entries.len() < 5 {
            return patterns;
        }

        // Group kills by hour of day
        let mut hourly_kills: std::collections::HashMap<u32, usize> =
            std::collections::HashMap::new();
        for entry in &self.entries {
            let hour = entry.killed_at.hour();
            *hourly_kills.entry(hour).or_insert(0) += 1;
        }

        // Find peak hours
        if let Some((peak_hour, count)) = hourly_kills.iter().max_by_key(|(_, &count)| count) {
            if *count > 2 {
                patterns.push(WorkflowPattern {
                    pattern_type: PatternType::TimeBased,
                    description: format!("Most kills happen around {}:00", peak_hour),
                    affected_processes: vec!["All processes".to_string()],
                    frequency: format!("{} kills at this hour", count),
                    recommendation: "Consider scheduling development work or adding processes to ignore list during peak hours.".to_string(),
                    confidence: 0.7,
                });
            }
        }

        patterns
    }

    /// Generate process management recommendations
    fn generate_process_management_recommendations(&self) -> Vec<SmartRecommendation> {
        let mut recommendations = Vec::new();

        // Check for frequently killed processes
        let frequent_offenders = self.get_frequent_offenders(2);
        if !frequent_offenders.is_empty() {
            recommendations.push(SmartRecommendation {
                category: RecommendationCategory::ProcessManagement,
                title: "Add Frequent Offenders to Ignore List".to_string(),
                description: format!(
                    "{} processes are being killed repeatedly",
                    frequent_offenders.len()
                ),
                action: "Use --ignore-processes flag to prevent repeated kills".to_string(),
                impact: "Reduces manual intervention and improves workflow efficiency".to_string(),
                priority: RecommendationPriority::High,
            });
        }

        recommendations
    }

    /// Generate port optimization recommendations
    fn generate_port_optimization_recommendations(&self) -> Vec<SmartRecommendation> {
        let mut recommendations = Vec::new();

        // Check for port conflicts
        let conflicts = self.analyze_port_conflicts();
        if !conflicts.is_empty() {
            recommendations.push(SmartRecommendation {
                category: RecommendationCategory::PortOptimization,
                title: "Resolve Port Conflicts".to_string(),
                description: format!("{} ports have conflicting processes", conflicts.len()),
                action: "Use different ports for different services or add conflicting ports to ignore list".to_string(),
                impact: "Prevents port binding errors and improves service reliability".to_string(),
                priority: RecommendationPriority::Medium,
            });
        }

        recommendations
    }

    /// Generate workflow improvement recommendations
    fn generate_workflow_recommendations(&self) -> Vec<SmartRecommendation> {
        let mut recommendations = Vec::new();

        // Check for auto-restart patterns
        let auto_restart_conflicts = self.analyze_auto_restart_patterns();
        if !auto_restart_conflicts.is_empty() {
            recommendations.push(SmartRecommendation {
                category: RecommendationCategory::WorkflowImprovement,
                title: "Handle Auto-Restart Processes".to_string(),
                description: "Some processes automatically restart after being killed".to_string(),
                action: "Investigate why processes restart or add them to ignore list".to_string(),
                impact: "Reduces frustration and improves development workflow".to_string(),
                priority: RecommendationPriority::Medium,
            });
        }

        recommendations
    }

    /// Generate analysis summary
    fn generate_analysis_summary(
        &self,
        conflicts: &[ProcessConflict],
        patterns: &[WorkflowPattern],
        recommendations: &[SmartRecommendation],
    ) -> String {
        let total_kills = self.entries.len();
        let conflict_count = conflicts.len();
        let pattern_count = patterns.len();
        let recommendation_count = recommendations.len();

        if total_kills == 0 {
            "No process history available for analysis.".to_string()
        } else if conflict_count == 0 && pattern_count == 0 {
            "Your development workflow appears to be running smoothly with no significant conflicts or patterns detected.".to_string()
        } else {
            format!(
                "Analysis of {} process kills revealed {} conflicts, {} workflow patterns, and {} recommendations for improvement.",
                total_kills, conflict_count, pattern_count, recommendation_count
            )
        }
    }
}
