use crate::types::ProcessInfo;
use std::collections::HashSet;
use regex::Regex;
use anyhow::Result;

pub struct SmartFilter {
    ignore_ports: HashSet<u16>,
    ignore_processes: HashSet<String>,
    ignore_patterns: Vec<Regex>,
    ignore_groups: HashSet<String>,
    only_groups: Option<HashSet<String>>,
}

impl SmartFilter {
    pub fn new(
        ignore_ports: HashSet<u16>,
        ignore_processes: HashSet<String>,
        ignore_patterns: Option<Vec<String>>,
        ignore_groups: HashSet<String>,
        only_groups: Option<HashSet<String>>,
    ) -> Result<Self> {
        let mut compiled_patterns = Vec::new();
        
        if let Some(patterns) = ignore_patterns {
            for pattern in patterns {
                // Convert wildcard pattern to regex
                let regex_pattern = pattern
                    .replace("*", ".*")
                    .replace("?", ".");
                let regex = Regex::new(&format!("^{}$", regex_pattern))?;
                compiled_patterns.push(regex);
            }
        }

        Ok(Self {
            ignore_ports,
            ignore_processes,
            ignore_patterns: compiled_patterns,
            ignore_groups,
            only_groups,
        })
    }

    pub fn should_ignore_process(&self, process_info: &ProcessInfo) -> bool {
        // Check port ignore list
        if self.ignore_ports.contains(&process_info.port) {
            return true;
        }

        // Check process name ignore list
        if self.ignore_processes.contains(&process_info.name) {
            return true;
        }

        // Check pattern matching against both name and command
        for pattern in &self.ignore_patterns {
            if pattern.is_match(&process_info.name) || pattern.is_match(&process_info.command) {
                return true;
            }
        }

        // Check group ignore list
        if let Some(ref group) = process_info.process_group {
            if self.ignore_groups.contains(group) {
                return true;
            }
        }

        // Check only_groups filter (if specified, only show these groups)
        if let Some(ref only_groups) = self.only_groups {
            if let Some(ref group) = process_info.process_group {
                if !only_groups.contains(group) {
                    return true;
                }
            } else {
                // Process has no group but only_groups is specified
                return true;
            }
        }

        false
    }

    pub fn filter_processes(&self, processes: &mut std::collections::HashMap<u16, ProcessInfo>) {
        processes.retain(|_, process_info| !self.should_ignore_process(process_info));
    }

    pub fn get_filter_stats(&self) -> FilterStats {
        FilterStats {
            ignore_ports_count: self.ignore_ports.len(),
            ignore_processes_count: self.ignore_processes.len(),
            ignore_patterns_count: self.ignore_patterns.len(),
            ignore_groups_count: self.ignore_groups.len(),
            only_groups_count: self.only_groups.as_ref().map_or(0, |g| g.len()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterStats {
    pub ignore_ports_count: usize,
    pub ignore_processes_count: usize,
    pub ignore_patterns_count: usize,
    pub ignore_groups_count: usize,
    pub only_groups_count: usize,
}

impl FilterStats {
    pub fn is_active(&self) -> bool {
        self.ignore_ports_count > 0 
            || self.ignore_processes_count > 0 
            || self.ignore_patterns_count > 0 
            || self.ignore_groups_count > 0 
            || self.only_groups_count > 0
    }

    pub fn get_description(&self) -> String {
        let mut parts = Vec::new();
        
        if self.ignore_ports_count > 0 {
            parts.push(format!("{} ports", self.ignore_ports_count));
        }
        if self.ignore_processes_count > 0 {
            parts.push(format!("{} processes", self.ignore_processes_count));
        }
        if self.ignore_patterns_count > 0 {
            parts.push(format!("{} patterns", self.ignore_patterns_count));
        }
        if self.ignore_groups_count > 0 {
            parts.push(format!("{} groups", self.ignore_groups_count));
        }
        if self.only_groups_count > 0 {
            parts.push(format!("{} only-groups", self.only_groups_count));
        }

        if parts.is_empty() {
            "no filters".to_string()
        } else {
            format!("filtering: {}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_pattern_matching() {
        let filter = SmartFilter::new(
            HashSet::new(),
            HashSet::new(),
            Some(vec!["node*".to_string(), "python*".to_string()]),
            HashSet::new(),
            None,
        ).unwrap();

        let mut processes = HashMap::new();
        processes.insert(3000, ProcessInfo {
            pid: 1234,
            port: 3000,
            command: "node".to_string(),
            name: "node".to_string(),
            container_id: None,
            container_name: None,
            command_line: None,
            working_directory: None,
            process_group: None,
            project_name: None,
        });

        processes.insert(8000, ProcessInfo {
            pid: 5678,
            port: 8000,
            command: "python".to_string(),
            name: "python".to_string(),
            container_id: None,
            container_name: None,
            command_line: None,
            working_directory: None,
            process_group: None,
            project_name: None,
        });

        processes.insert(9000, ProcessInfo {
            pid: 9012,
            port: 9000,
            command: "rust".to_string(),
            name: "rust".to_string(),
            container_id: None,
            container_name: None,
            command_line: None,
            working_directory: None,
            process_group: None,
            project_name: None,
        });

        filter.filter_processes(&mut processes);
        
        // Only rust should remain (node and python should be filtered out)
        assert_eq!(processes.len(), 1);
        assert!(processes.contains_key(&9000));
    }

    #[test]
    fn test_only_groups_filter() {
        let filter = SmartFilter::new(
            HashSet::new(),
            HashSet::new(),
            None,
            HashSet::new(),
            Some(["Node.js".to_string()].iter().cloned().collect()),
        ).unwrap();

        let mut processes = HashMap::new();
        processes.insert(3000, ProcessInfo {
            pid: 1234,
            port: 3000,
            command: "node".to_string(),
            name: "node".to_string(),
            container_id: None,
            container_name: None,
            command_line: None,
            working_directory: None,
            process_group: Some("Node.js".to_string()),
            project_name: None,
        });

        processes.insert(8000, ProcessInfo {
            pid: 5678,
            port: 8000,
            command: "python".to_string(),
            name: "python".to_string(),
            container_id: None,
            container_name: None,
            command_line: None,
            working_directory: None,
            process_group: Some("Python".to_string()),
            project_name: None,
        });

        filter.filter_processes(&mut processes);
        
        // Only Node.js should remain
        assert_eq!(processes.len(), 1);
        assert!(processes.contains_key(&3000));
    }
}
