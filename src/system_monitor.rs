use sysinfo::{System, Pid};
use std::collections::HashMap;

pub struct SystemMonitor {
    system: System,
    last_cpu_times: HashMap<Pid, f64>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            last_cpu_times: HashMap::new(),
        }
    }
    
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    pub fn get_process_cpu_usage(&mut self, pid: i32) -> Option<f64> {
        let pid = Pid::from_u32(pid as u32);
        
        if let Some(process) = self.system.process(pid) {
            let current_cpu_time = process.cpu_usage() as f64;
            
            // Calculate CPU usage since last check
            let cpu_usage = if let Some(last_time) = self.last_cpu_times.get(&pid) {
                let delta = current_cpu_time - last_time;
                // Cap at 100% and ensure non-negative
                delta.min(100.0).max(0.0)
            } else {
                // First time seeing this process, return current usage
                current_cpu_time.min(100.0).max(0.0)
            };
            
            // Store current time for next calculation
            self.last_cpu_times.insert(pid, current_cpu_time);
            
            Some(cpu_usage)
        } else {
            // Process not found, remove from tracking
            self.last_cpu_times.remove(&pid);
            None
        }
    }
    
    pub fn get_process_memory_usage(&self, pid: i32) -> Option<(u64, f64)> {
        let pid = Pid::from_u32(pid as u32);
        
        if let Some(process) = self.system.process(pid) {
            let memory_bytes = process.memory() * 1024; // Convert from KB to bytes
            let total_memory = self.system.total_memory() * 1024; // Convert from KB to bytes
            let memory_percentage = if total_memory > 0 {
                (memory_bytes as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            };
            
            Some((memory_bytes, memory_percentage))
        } else {
            None
        }
    }
    
    pub fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            total_memory: self.system.total_memory() * 1024, // Convert to bytes
            used_memory: self.system.used_memory() * 1024,   // Convert to bytes
            total_swap: self.system.total_swap() * 1024,     // Convert to bytes
            used_swap: self.system.used_swap() * 1024,       // Convert to bytes
            cpu_count: self.system.cpus().len(),
            load_average: sysinfo::System::load_average(),
        }
    }
    
    pub fn cleanup_old_processes(&mut self) {
        // Remove processes that are no longer running
        let current_pids: std::collections::HashSet<Pid> = self.system.processes()
            .iter()
            .map(|(pid, _)| *pid)
            .collect();
        
        self.last_cpu_times.retain(|pid, _| current_pids.contains(pid));
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub cpu_count: usize,
    pub load_average: sysinfo::LoadAvg,
}

impl SystemInfo {
    pub fn memory_percentage(&self) -> f64 {
        if self.total_memory > 0 {
            (self.used_memory as f64 / self.total_memory as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn swap_percentage(&self) -> f64 {
        if self.total_swap > 0 {
            (self.used_swap as f64 / self.total_swap as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn format_memory(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        const THRESHOLD: u64 = 1024;
        
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD as f64;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_formatting() {
        let system_info = SystemInfo {
            total_memory: 0,
            used_memory: 0,
            total_swap: 0,
            used_swap: 0,
            cpu_count: 4,
            load_average: sysinfo::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 },
        };
        
        assert_eq!(system_info.format_memory(1024), "1.0 KB");
        assert_eq!(system_info.format_memory(1048576), "1.0 MB");
        assert_eq!(system_info.format_memory(1073741824), "1.0 GB");
    }
}
