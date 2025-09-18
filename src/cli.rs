use clap::Parser;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum LogLevel {
    /// Show all logs (info, warn, error)
    Info,
    /// Show only warning and error logs
    Warn,
    /// Show only error logs
    Error,
    /// Show no logs
    None,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "port-kill",
    about = "A lightweight macOS status bar app that monitors and manages development processes",
    version,
    long_about = "Monitors development processes running on specified ports and allows you to kill them from the status bar."
)]
pub struct Args {
    /// Starting port for range scanning (inclusive)
    #[arg(short, long, default_value = "2000")]
    pub start_port: u16,

    /// Ending port for range scanning (inclusive)
    #[arg(short, long, default_value = "6000")]
    pub end_port: u16,

    /// Specific ports to monitor (comma-separated, supports ranges like 3000-3010, overrides start/end port range)
    #[arg(short, long, value_delimiter = ',')]
    pub ports: Option<Vec<String>>,

    /// Ports to ignore (comma-separated, e.g., 5353,5000,7000 for Chromecast/AirDrop)
    #[arg(long, value_delimiter = ',')]
    pub ignore_ports: Option<Vec<u16>>,

    /// Process names to ignore (comma-separated, e.g., Chrome,ControlCe)
    #[arg(long, value_delimiter = ',')]
    pub ignore_processes: Option<Vec<String>>,

    /// Process name patterns to ignore (supports wildcards: *, ?)
    #[arg(long, value_delimiter = ',')]
    pub ignore_patterns: Option<Vec<String>>,

    /// Process groups to ignore (e.g., Database,Web Server)
    #[arg(long, value_delimiter = ',')]
    pub ignore_groups: Option<Vec<String>>,

    /// Enable smart filtering (auto-detect and ignore system processes)
    #[arg(long)]
    pub smart_filter: bool,

    /// Only show processes from specific groups (e.g., Node.js,Python)
    #[arg(long, value_delimiter = ',')]
    pub only_groups: Option<Vec<String>>,

    /// Run in console mode instead of status bar mode
    #[arg(short, long)]
    pub console: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Enable Docker container monitoring (includes containers in process detection)
    #[arg(short, long)]
    pub docker: bool,

    /// Show process IDs (PIDs) in the display output
    #[arg(short = 'P', long)]
    pub show_pid: bool,

    /// Log level (info, warn, error, none)
    #[arg(long, default_value = "info", value_enum)]
    pub log_level: LogLevel,

    /// Show process kill history
    #[arg(long)]
    pub show_history: bool,

    /// Clear process kill history
    #[arg(long)]
    pub clear_history: bool,

    /// Show filter configuration
    #[arg(long)]
    pub show_filters: bool,

    /// Enable performance metrics (CPU and memory usage)
    #[arg(long)]
    pub performance: bool,

    /// Show project context for each process
    #[arg(long)]
    pub show_context: bool,

    /// Kill all processes on the specified ports
    #[arg(long)]
    pub kill_all: bool,

    /// Kill processes by group (e.g., Node.js, Python)
    #[arg(long, value_delimiter = ',')]
    pub kill_group: Option<Vec<String>>,

    /// Kill processes by project name
    #[arg(long, value_delimiter = ',')]
    pub kill_project: Option<Vec<String>>,

    /// Restart processes (kill and wait for them to restart)
    #[arg(long)]
    pub restart: bool,

    /// Show process tree (parent-child relationships)
    #[arg(long)]
    pub show_tree: bool,

    /// Output processes in JSON format (for API integration)
    #[arg(long)]
    pub json: bool,

    /// Reset common development ports (3000, 5000, 8000, 5432, 3306, 6379, 27017, 8080, 9000)
    #[arg(long)]
    pub reset: bool,

    /// Show frequent offenders (processes killed multiple times)
    #[arg(long)]
    pub show_offenders: bool,

    /// Show time patterns and statistics
    #[arg(long)]
    pub show_patterns: bool,

    /// Show auto-suggestions for ignore lists
    #[arg(long)]
    pub show_suggestions: bool,

    /// Show detailed history statistics
    #[arg(long)]
    pub show_stats: bool,

    /// Show smart root cause analysis
    #[arg(long)]
    pub show_root_cause: bool,

    /// Enable Port Guard Mode - proactive port conflict prevention
    #[arg(long)]
    pub guard_mode: bool,

    /// Ports to watch in guard mode (comma-separated)
    #[arg(long, default_value = "3000,3001,3002,8000,8080,9000")]
    pub guard_ports: String,

    /// Auto-resolve conflicts by killing conflicting processes
    #[arg(long)]
    pub auto_resolve: bool,

    /// Port reservation file path for persistent port assignments
    #[arg(long, default_value = "~/.port-kill/reservations.json")]
    pub reservation_file: String,

    /// Enable process interception for development commands
    #[arg(long)]
    pub intercept_commands: bool,

    /// Reserve a port for a specific project (requires guard mode)
    #[arg(long)]
    pub reserve_port: Option<u16>,

    /// Project name for port reservation
    #[arg(long)]
    pub project_name: Option<String>,

    /// Process name for port reservation
    #[arg(long)]
    pub process_name: Option<String>,

    /// Enable Security Audit Mode - comprehensive security analysis
    #[arg(long)]
    pub audit: bool,

    /// Security mode with enhanced suspicious port detection
    #[arg(long)]
    pub security_mode: bool,

    /// Suspicious ports to flag (comma-separated)
    #[arg(long, default_value = "8444,4444,9999,14444,5555,6666,7777")]
    pub suspicious_ports: String,

    /// Baseline file for approved ports comparison
    #[arg(long)]
    pub baseline_file: Option<String>,

    /// Show only suspicious/unauthorized processes
    #[arg(long)]
    pub suspicious_only: bool,

    /// Remote mode: connect to remote host via SSH
    #[arg(long)]
    pub remote: Option<String>,
}

impl Args {
    /// Get the list of ports to monitor
    pub fn get_ports_to_monitor(&self) -> Vec<u16> {
        if let Some(ref specific_ports) = self.ports {
            // Parse port strings (supports individual ports and ranges like "3000-3010")
            let mut ports = Vec::new();
            for port_str in specific_ports {
                if let Some(range_ports) = self.parse_port_range(port_str) {
                    ports.extend(range_ports);
                }
            }
            ports
        } else {
            // Use port range
            (self.start_port..=self.end_port).collect()
        }
    }

    /// Parse a port string that can be either a single port or a range (e.g., "3000" or "3000-3010")
    fn parse_port_range(&self, port_str: &str) -> Option<Vec<u16>> {
        let port_str = port_str.trim();
        
        if port_str.contains('-') {
            // Handle port range (e.g., "3000-3010")
            let parts: Vec<&str> = port_str.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                    if start <= end {
                        return Some((start..=end).collect());
                    }
                }
            }
            None
        } else {
            // Handle single port
            port_str.parse::<u16>().ok().map(|port| vec![port])
        }
    }

    /// Get a HashSet of ports for efficient lookup
    pub fn get_ports_set(&self) -> HashSet<u16> {
        self.get_ports_to_monitor().into_iter().collect()
    }

    /// Get a HashSet of ports to ignore for efficient lookup
    pub fn get_ignore_ports_set(&self) -> HashSet<u16> {
        self.ignore_ports.clone().unwrap_or_default().into_iter().collect()
    }

    /// Get a HashSet of process names to ignore for efficient lookup
    pub fn get_ignore_processes_set(&self) -> HashSet<String> {
        self.ignore_processes.clone().unwrap_or_default().into_iter().collect()
    }

    /// Get a HashSet of process groups to ignore for efficient lookup
    pub fn get_ignore_groups_set(&self) -> HashSet<String> {
        self.ignore_groups.clone().unwrap_or_default().into_iter().collect()
    }

    /// Get a HashSet of process groups to show (if only_groups is specified)
    pub fn get_only_groups_set(&self) -> Option<HashSet<String>> {
        self.only_groups.as_ref().map(|groups| groups.iter().cloned().collect())
    }

    /// Get smart filter defaults
    pub fn get_smart_filter_defaults(&self) -> (HashSet<u16>, HashSet<String>, HashSet<String>) {
        if !self.smart_filter {
            return (HashSet::new(), HashSet::new(), HashSet::new());
        }

        // Smart port ignores (common system ports)
        let smart_ignore_ports: HashSet<u16> = [
            22,    // SSH
            25,    // SMTP
            53,    // DNS
            80,    // HTTP (system)
            443,   // HTTPS (system)
            993,   // IMAPS
            995,   // POP3S
            1433,  // SQL Server
            3306,  // MySQL
            5432,  // PostgreSQL
            6379,  // Redis
            27017, // MongoDB
            5353,  // mDNS/Bonjour
            5000,  // AirDrop
            7000,  // AirDrop
            8080,  // Common proxy
            8443,  // HTTPS Alt
        ].iter().cloned().collect();

        // Smart process ignores (common system processes)
        let smart_ignore_processes: HashSet<String> = [
            "Chrome", "Safari", "Firefox", "Edge", // Browsers
            "ControlCe", "rapportd", "AirPlayXP", // macOS system
            "systemd", "init", "kthreadd", // Linux system
            "svchost", "explorer", "winlogon", // Windows system
            "docker", "dockerd", "containerd", // Docker system
            "nginx", "apache2", "httpd", // Web servers (system)
            "mysqld", "postgres", "redis-server", // Database servers
            "ssh", "sshd", // SSH servers
        ].iter().map(|s| s.to_string()).collect();

        // Smart group ignores (system service groups)
        let smart_ignore_groups: HashSet<String> = [
            "Web Server", // System web servers
            "Database",   // System databases
        ].iter().map(|s| s.to_string()).collect();

        (smart_ignore_ports, smart_ignore_processes, smart_ignore_groups)
    }

    /// Get a description of the port configuration
    pub fn get_port_description(&self) -> String {
        let mut description = if let Some(ref specific_ports) = self.ports {
            format!("specific ports: {}", specific_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "))
        } else {
            format!("port range: {}-{}", self.start_port, self.end_port)
        };

        // Add ignore information to description
        let mut ignore_info = Vec::new();
        
        if let Some(ref ignore_ports) = self.ignore_ports {
            if !ignore_ports.is_empty() {
                ignore_info.push(format!("ignoring ports: {}", ignore_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")));
            }
        }
        
        if let Some(ref ignore_processes) = self.ignore_processes {
            if !ignore_processes.is_empty() {
                ignore_info.push(format!("ignoring processes: {}", ignore_processes.join(", ")));
            }
        }
        
        if !ignore_info.is_empty() {
            description.push_str(&format!(" ({})", ignore_info.join(", ")));
        }
        
        description
    }

    /// Validate the arguments
    pub fn validate(&self) -> Result<(), String> {
        // Validate port range
        if self.start_port > self.end_port {
            return Err("Start port cannot be greater than end port".to_string());
        }

        // Validate specific ports if provided
        if let Some(ref specific_ports) = self.ports {
            if specific_ports.is_empty() {
                return Err("At least one port must be specified".to_string());
            }
            
            for port_str in specific_ports {
                if let Some(ports) = self.parse_port_range(port_str) {
                    for port in ports {
                        if port == 0 {
                            return Err("Port 0 is not valid".to_string());
                        }
                    }
                } else {
                    return Err(format!("Invalid port specification: '{}'", port_str));
                }
            }
        }

        // Validate ignore ports if provided
        if let Some(ref ignore_ports) = self.ignore_ports {
            for &port in ignore_ports {
                if port == 0 {
                    return Err("Ignore port 0 is not valid".to_string());
                }
            }
        }

        // Validate ignore processes if provided
        if let Some(ref ignore_processes) = self.ignore_processes {
            for process_name in ignore_processes {
                if process_name.trim().is_empty() {
                    return Err("Ignore process names cannot be empty".to_string());
                }
            }
        }

        Ok(())
    }

    /// Get common development ports for reset functionality
    pub fn get_reset_ports(&self) -> Vec<u16> {
        vec![
            3000,   // React, Next.js, Node.js dev servers
            5000,   // Flask, Express, various dev servers
            8000,   // Django, Rails, various dev servers
            5432,   // PostgreSQL
            3306,   // MySQL
            6379,   // Redis
            27017,  // MongoDB
            8080,   // Tomcat, various Java apps
            9000,   // Various dev servers
        ]
    }

    /// Get the list of ports to watch in guard mode
    pub fn get_guard_ports(&self) -> Vec<u16> {
        self.guard_ports
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }

    /// Get the expanded reservation file path
    pub fn get_reservation_file_path(&self) -> String {
        if self.reservation_file.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            self.reservation_file.replace("~/", &format!("{}/", home))
        } else {
            self.reservation_file.clone()
        }
    }

    /// Get the list of suspicious ports
    pub fn get_suspicious_ports(&self) -> Vec<u16> {
        self.suspicious_ports
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }

    /// Get the expanded baseline file path
    pub fn get_baseline_file_path(&self) -> Option<String> {
        self.baseline_file.as_ref().map(|path| {
            if path.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                path.replace("~/", &format!("{}/", home))
            } else {
                path.clone()
            }
        })
    }

    /// Get the remote host for SSH connection
    pub fn get_remote_host(&self) -> Option<String> {
        self.remote.clone()
    }
}

impl LogLevel {
    /// Convert LogLevel to RUST_LOG environment variable value
    pub fn to_rust_log(&self) -> &'static str {
        match self {
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::None => "off",
        }
    }

    /// Check if info level logging is enabled
    pub fn is_info_enabled(&self) -> bool {
        matches!(self, LogLevel::Info)
    }

    /// Check if warn level logging is enabled
    pub fn is_warn_enabled(&self) -> bool {
        matches!(self, LogLevel::Info | LogLevel::Warn)
    }

    /// Check if error level logging is enabled
    pub fn is_error_enabled(&self) -> bool {
        matches!(self, LogLevel::Info | LogLevel::Warn | LogLevel::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ports_to_monitor_range() {
        let args = Args {
            start_port: 3000,
            end_port: 3005,
            ports: None,
            ignore_ports: None,
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        let ports = args.get_ports_to_monitor();
        assert_eq!(ports, vec![3000, 3001, 3002, 3003, 3004, 3005]);
    }

    #[test]
    fn test_get_ports_to_monitor_specific() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: Some(vec!["3000".to_string(), "8000".to_string(), "8080".to_string()]),
            ignore_ports: None,
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        let ports = args.get_ports_to_monitor();
        assert_eq!(ports, vec![3000, 8000, 8080]);
    }

    #[test]
    fn test_get_ports_to_monitor_with_ranges() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: Some(vec!["3000-3002".to_string(), "8000".to_string(), "8080-8081".to_string()]),
            ignore_ports: None,
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        let ports = args.get_ports_to_monitor();
        assert_eq!(ports, vec![3000, 3001, 3002, 8000, 8080, 8081]);
    }

    #[test]
    fn test_get_ignore_ports_set() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: None,
            ignore_ports: Some(vec![5353, 5000, 7000]),
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        let ignore_ports = args.get_ignore_ports_set();
        assert_eq!(ignore_ports, HashSet::from([5353, 5000, 7000]));
    }

    #[test]
    fn test_get_ignore_processes_set() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: None,
            ignore_ports: None,
            ignore_processes: Some(vec!["Chrome".to_string(), "ControlCe".to_string()]),
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        let ignore_processes = args.get_ignore_processes_set();
        assert_eq!(ignore_processes, HashSet::from([String::from("Chrome"), String::from("ControlCe")]));
    }

    #[test]
    fn test_get_port_description_with_ignores() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: None,
            ignore_ports: Some(vec![5353, 5000]),
            ignore_processes: Some(vec!["Chrome".to_string(), "ControlCe".to_string()]),
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        assert_eq!(args.get_port_description(), "port range: 2000-6000 (ignoring ports: 5353, 5000, ignoring processes: Chrome, ControlCe)");
    }

    #[test]
    fn test_get_port_description_range() {
        let args = Args {
            start_port: 3000,
            end_port: 3010,
            ports: None,
            ignore_ports: None,
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        assert_eq!(args.get_port_description(), "port range: 3000-3010");
    }

    #[test]
    fn test_get_port_description_specific() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: Some(vec!["3000".to_string(), "8000".to_string(), "8080".to_string()]),
            ignore_ports: None,
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        assert_eq!(args.get_port_description(), "specific ports: 3000, 8000, 8080");
    }

    #[test]
    fn test_validation_valid() {
        let args = Args {
            start_port: 3000,
            end_port: 3010,
            ports: None,
            ignore_ports: None,
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_range() {
        let args = Args {
            start_port: 3010,
            end_port: 3000,
            ports: None,
            ignore_ports: None,
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validation_empty_specific_ports() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: Some(vec![]),
            ignore_ports: None,
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_ignore_port() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: None,
            ignore_ports: Some(vec![0]),
            ignore_processes: None,
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validation_empty_ignore_process() {
        let args = Args {
            start_port: 2000,
            end_port: 6000,
            ports: None,
            ignore_ports: None,
            ignore_processes: Some(vec!["".to_string()]),
            console: false,
            verbose: false,
            docker: false,
            show_pid: false,
        };
        
        assert!(args.validate().is_err());
    }
}
