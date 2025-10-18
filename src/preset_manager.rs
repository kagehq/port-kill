use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a port preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortPreset {
    /// Name of the preset
    pub name: String,
    /// Description of what this preset is for
    pub description: String,
    /// List of ports to monitor
    pub ports: Vec<u16>,
    /// Ports to ignore (comma-separated)
    pub ignore_ports: Option<Vec<u16>>,
    /// Process names to ignore (comma-separated)
    pub ignore_processes: Option<Vec<String>>,
    /// Process name patterns to ignore (supports wildcards: *, ?)
    pub ignore_patterns: Option<Vec<String>>,
    /// Process groups to ignore
    pub ignore_groups: Option<Vec<String>>,
    /// Only show processes from specific groups
    pub only_groups: Option<Vec<String>>,
    /// Enable smart filtering
    pub smart_filter: bool,
    /// Enable Docker container monitoring
    pub docker: bool,
    /// Show process IDs
    pub show_pid: bool,
    /// Enable performance metrics
    pub performance: bool,
    /// Show project context
    pub show_context: bool,
}

impl PortPreset {
    /// Create a new preset
    pub fn new(name: String, description: String, ports: Vec<u16>) -> Self {
        Self {
            name,
            description,
            ports,
            ignore_ports: None,
            ignore_processes: None,
            ignore_patterns: None,
            ignore_groups: None,
            only_groups: None,
            smart_filter: false,
            docker: false,
            show_pid: false,
            performance: false,
            show_context: false,
        }
    }

    /// Create a preset with ignore settings
    pub fn with_ignores(
        name: String,
        description: String,
        ports: Vec<u16>,
        ignore_ports: Option<Vec<u16>>,
        ignore_processes: Option<Vec<String>>,
        ignore_patterns: Option<Vec<String>>,
        ignore_groups: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            description,
            ports,
            ignore_ports,
            ignore_processes,
            ignore_patterns,
            ignore_groups,
            only_groups: None,
            smart_filter: false,
            docker: false,
            show_pid: false,
            performance: false,
            show_context: false,
        }
    }

    /// Create a preset with smart filtering enabled
    pub fn with_smart_filter(
        name: String,
        description: String,
        ports: Vec<u16>,
        smart_filter: bool,
    ) -> Self {
        Self {
            name,
            description,
            ports,
            ignore_ports: None,
            ignore_processes: None,
            ignore_patterns: None,
            ignore_groups: None,
            only_groups: None,
            smart_filter,
            docker: false,
            show_pid: false,
            performance: false,
            show_context: false,
        }
    }
}

/// Manages port presets
pub struct PresetManager {
    presets: HashMap<String, PortPreset>,
    config_path: String,
}

impl PresetManager {
    /// Create a new preset manager
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let config_path = format!("{}/.port-kill/presets.json", home);

        Self {
            presets: HashMap::new(),
            config_path,
        }
    }

    /// Load presets from file
    pub fn load_presets(&mut self) -> Result<()> {
        // First, load default presets
        self.load_default_presets();

        // Then try to load user presets from file
        if Path::new(&self.config_path).exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let user_presets: HashMap<String, PortPreset> = serde_json::from_str(&content)?;

            // Merge user presets (they override defaults)
            for (name, preset) in user_presets {
                self.presets.insert(name, preset);
            }
        }

        Ok(())
    }

    /// Save presets to file
    pub fn save_presets(&self) -> Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&self.config_path).parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.presets)?;
        fs::write(&self.config_path, content)?;

        Ok(())
    }

    /// Load default presets
    fn load_default_presets(&mut self) {
        // Development preset - common dev ports
        let dev_preset = PortPreset::with_ignores(
            "dev".to_string(),
            "Common development ports (React, Node.js, Python, etc.)".to_string(),
            vec![3000, 3001, 3002, 4321, 5000, 8000, 8080, 9000],
            Some(vec![5353, 7000]), // Ignore AirDrop and mDNS
            Some(vec![
                "Chrome".to_string(),
                "Safari".to_string(),
                "Firefox".to_string(),
            ]), // Ignore browsers
            None,
            None,
        );
        self.presets.insert("dev".to_string(), dev_preset);

        // System preset - system services
        let system_preset = PortPreset::with_smart_filter(
            "system".to_string(),
            "System services and daemons".to_string(),
            vec![
                22, 25, 53, 80, 443, 993, 995, 1433, 3306, 5432, 6379, 27017, 8080, 8443,
            ],
            true, // Enable smart filtering
        );
        self.presets.insert("system".to_string(), system_preset);

        // Database preset - database services
        let db_preset = PortPreset::new(
            "database".to_string(),
            "Database services (MySQL, PostgreSQL, Redis, MongoDB, etc.)".to_string(),
            vec![3306, 5432, 6379, 27017, 1433, 9200, 9300],
        );
        self.presets.insert("database".to_string(), db_preset);

        // Web preset - web servers and proxies
        let web_preset = PortPreset::with_ignores(
            "web".to_string(),
            "Web servers and proxies".to_string(),
            vec![80, 443, 8080, 8443, 3000, 5000, 8000, 9000],
            Some(vec![5353]), // Ignore mDNS
            Some(vec![
                "nginx".to_string(),
                "apache2".to_string(),
                "httpd".to_string(),
            ]), // Ignore system web servers
            None,
            None,
        );
        self.presets.insert("web".to_string(), web_preset);

        // React preset - React development
        let react_preset = PortPreset::new(
            "react".to_string(),
            "React development servers".to_string(),
            vec![3000, 3001, 3002, 3003, 3004, 3005],
        );
        self.presets.insert("react".to_string(), react_preset);

        // Node.js preset - Node.js development
        let node_preset = PortPreset::new(
            "node".to_string(),
            "Node.js development servers".to_string(),
            vec![3000, 5000, 8000, 8080, 9000],
        );
        self.presets.insert("node".to_string(), node_preset);

        // Python preset - Python development
        let python_preset = PortPreset::new(
            "python".to_string(),
            "Python development servers (Django, Flask, FastAPI, etc.)".to_string(),
            vec![5000, 8000, 8080, 9000],
        );
        self.presets.insert("python".to_string(), python_preset);

        // Full range preset - comprehensive monitoring
        let full_preset = PortPreset::with_smart_filter(
            "full".to_string(),
            "Comprehensive port monitoring (2000-8000 with smart filtering)".to_string(),
            (2000..=8000).collect(),
            true, // Enable smart filtering
        );
        self.presets.insert("full".to_string(), full_preset);

        // Minimal preset - just the essentials
        let minimal_preset = PortPreset::new(
            "minimal".to_string(),
            "Essential development ports only".to_string(),
            vec![3000, 8080, 4321],
        );
        self.presets.insert("minimal".to_string(), minimal_preset);
    }

    /// Get a preset by name
    pub fn get_preset(&self, name: &str) -> Option<&PortPreset> {
        self.presets.get(name)
    }

    /// Get all preset names
    pub fn get_preset_names(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }

    /// Add or update a preset
    pub fn add_preset(&mut self, preset: PortPreset) {
        self.presets.insert(preset.name.clone(), preset);
    }

    /// Remove a preset
    pub fn remove_preset(&mut self, name: &str) -> Option<PortPreset> {
        self.presets.remove(name)
    }

    /// List all presets with descriptions
    pub fn list_presets(&self) -> String {
        let mut output = String::new();
        output.push_str("📋 Available Port Presets:\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');

        let mut preset_names: Vec<_> = self.presets.keys().collect();
        preset_names.sort();

        for name in preset_names {
            if let Some(preset) = self.presets.get(name) {
                output.push_str(&format!("• {}: {}\n", name, preset.description));
                output.push_str(&format!(
                    "  Ports: {}\n",
                    preset
                        .ports
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));

                if let Some(ref ignore_ports) = preset.ignore_ports {
                    if !ignore_ports.is_empty() {
                        output.push_str(&format!(
                            "  Ignores ports: {}\n",
                            ignore_ports
                                .iter()
                                .map(|p| p.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }

                if let Some(ref ignore_processes) = preset.ignore_processes {
                    if !ignore_processes.is_empty() {
                        output.push_str(&format!(
                            "  Ignores processes: {}\n",
                            ignore_processes.join(", ")
                        ));
                    }
                }

                if preset.smart_filter {
                    output.push_str("  Smart filtering: enabled\n");
                }

                output.push('\n');
            }
        }

        output
    }

    /// Get the config file path
    pub fn get_config_path(&self) -> &str {
        &self.config_path
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_creation() {
        let preset = PortPreset::new(
            "test".to_string(),
            "Test preset".to_string(),
            vec![3000, 8080],
        );

        assert_eq!(preset.name, "test");
        assert_eq!(preset.description, "Test preset");
        assert_eq!(preset.ports, vec![3000, 8080]);
    }

    #[test]
    fn test_preset_with_ignores() {
        let preset = PortPreset::with_ignores(
            "test".to_string(),
            "Test preset".to_string(),
            vec![3000, 8080],
            Some(vec![5353]),
            Some(vec!["Chrome".to_string()]),
            None,
            None,
        );

        assert_eq!(preset.ignore_ports, Some(vec![5353]));
        assert_eq!(preset.ignore_processes, Some(vec!["Chrome".to_string()]));
    }

    #[test]
    fn test_preset_manager() {
        let mut manager = PresetManager::new();
        manager.load_default_presets();

        assert!(manager.get_preset("dev").is_some());
        assert!(manager.get_preset("system").is_some());
        assert!(manager.get_preset("nonexistent").is_none());

        let names = manager.get_preset_names();
        assert!(names.contains(&"dev".to_string()));
        assert!(names.contains(&"system".to_string()));
    }
}
