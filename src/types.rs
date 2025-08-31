use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: i32,
    pub port: u16,
    pub command: String,
    pub name: String,
    pub container_id: Option<String>,
    pub container_name: Option<String>,
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
}
