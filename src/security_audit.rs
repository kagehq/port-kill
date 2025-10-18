use anyhow::Result;
use chrono::Utc;
use log::info;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::{
    ApprovedProcess, BaselineComparison, ProcessChange, ProcessChangeType, ProcessInfo, RiskLevel,
    SecurityAuditResult, SecurityRecommendation, ServiceType, SuspicionReason, SuspiciousProcess,
};

/// Security Audit engine for comprehensive port and process analysis
pub struct SecurityAuditor {
    suspicious_ports: Vec<u16>,
    baseline_file: Option<String>,
    _suspicious_only: bool,
}

impl SecurityAuditor {
    /// Create a new Security Auditor
    pub fn new(
        suspicious_ports: Vec<u16>,
        baseline_file: Option<String>,
        suspicious_only: bool,
    ) -> Self {
        Self {
            suspicious_ports,
            baseline_file,
            _suspicious_only: suspicious_only,
        }
    }

    /// Perform comprehensive security audit
    pub async fn perform_audit(
        &self,
        processes: HashMap<u16, ProcessInfo>,
    ) -> Result<SecurityAuditResult> {
        info!(
            "🔒 Starting Security Audit - scanning {} processes",
            processes.len()
        );

        let mut suspicious_processes = Vec::new();
        let mut approved_processes = Vec::new();
        let mut recommendations = Vec::new();

        // Analyze each process with timeout protection
        for (port, process) in &processes {
            // Add timeout to prevent hanging on individual process analysis
            let analysis = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                self.analyze_process(*port, process),
            )
            .await
            .map_err(|_| anyhow::anyhow!("Process analysis timeout for port {}", port))?
            .map_err(|e| anyhow::anyhow!("Process analysis failed for port {}: {}", port, e))?;

            match analysis.risk_level {
                RiskLevel::Low => {
                    approved_processes.push(ApprovedProcess {
                        port: *port,
                        process_info: process.clone(),
                        service_type: self.classify_service_type(process),
                        expected_location: self.get_expected_location(process),
                        binary_hash: self.calculate_binary_hash(process).await.ok(),
                    });
                }
                _ => {
                    suspicious_processes.push(analysis);
                }
            }
        }

        // Generate security recommendations
        recommendations.extend(self.generate_recommendations(&suspicious_processes));

        // Calculate security score
        let security_score = self.calculate_security_score(&suspicious_processes, processes.len());

        // Perform baseline comparison if baseline file exists (with timeout)
        let baseline_comparison = if let Some(baseline_path) = &self.baseline_file {
            tokio::time::timeout(
                std::time::Duration::from_secs(10),
                self.compare_with_baseline(baseline_path, &processes),
            )
            .await
            .map_err(|_| {
                log::warn!("Baseline comparison timeout, skipping");
                anyhow::anyhow!("Baseline comparison timeout")
            })
            .ok()
            .and_then(|result| result.ok())
        } else {
            None
        };

        let result = SecurityAuditResult {
            audit_timestamp: Utc::now(),
            total_ports_scanned: processes.len(),
            suspicious_processes,
            approved_processes,
            security_score,
            recommendations,
            baseline_comparison,
        };

        info!(
            "🔒 Security Audit completed - Score: {:.1}/100",
            security_score
        );
        Ok(result)
    }

    /// Analyze a single process for security risks
    async fn analyze_process(&self, port: u16, process: &ProcessInfo) -> Result<SuspiciousProcess> {
        let mut suspicion_reasons = Vec::new();
        let mut risk_level = RiskLevel::Low;

        // Check for suspicious ports
        if self.suspicious_ports.contains(&port) {
            suspicion_reasons.push(SuspicionReason::SuspiciousPort);
            risk_level = RiskLevel::Critical;
        }

        // Check for unknown binaries
        if self.is_unknown_binary(process) {
            suspicion_reasons.push(SuspicionReason::UnknownBinary);
            risk_level = self.escalate_risk(risk_level);
        }

        // Check for unexpected locations
        if self.is_unexpected_location(process) {
            suspicion_reasons.push(SuspicionReason::UnexpectedLocation);
            risk_level = self.escalate_risk(risk_level);
        }

        // Check for high privilege processes
        if self.is_high_privilege(process) {
            suspicion_reasons.push(SuspicionReason::HighPrivilege);
            risk_level = self.escalate_risk(risk_level);
        }

        // Check for network exposure
        if self.has_network_exposure(process) {
            suspicion_reasons.push(SuspicionReason::NetworkExposure);
            risk_level = self.escalate_risk(risk_level);
        }

        // Check for process anomalies
        if self.has_process_anomaly(process) {
            suspicion_reasons.push(SuspicionReason::ProcessAnomaly);
            risk_level = self.escalate_risk(risk_level);
        }

        // Get primary suspicion reason (highest risk)
        let primary_reason = suspicion_reasons
            .first()
            .cloned()
            .unwrap_or(SuspicionReason::ProcessAnomaly);

        Ok(SuspiciousProcess {
            port,
            process_info: process.clone(),
            suspicion_reason: primary_reason,
            risk_level,
            binary_hash: self.calculate_binary_hash(process).await.ok(),
            parent_process: self.get_parent_process(process).await.ok(),
            network_interface: self.get_network_interface(process),
            first_seen: Utc::now(), // TODO: Get actual first seen time
        })
    }

    /// Check if binary is unknown/suspicious
    fn is_unknown_binary(&self, process: &ProcessInfo) -> bool {
        let suspicious_paths = [
            "/tmp/",
            "/var/tmp/",
            "/dev/shm/",
            "/proc/",
            "/home/",
            "/root/",
            "/opt/",
            "/usr/local/bin/",
        ];

        let suspicious_names = [
            "miner", "crypto", "bitcoin", "monero", "xmr", "backdoor", "shell", "reverse",
            "payload",
        ];

        let path = process.name.to_lowercase();
        let name = process.name.to_lowercase();

        // Check for suspicious paths
        if suspicious_paths
            .iter()
            .any(|&susp_path| path.contains(susp_path))
        {
            return true;
        }

        // Check for suspicious names
        if suspicious_names
            .iter()
            .any(|&susp_name| name.contains(susp_name))
        {
            return true;
        }

        false
    }

    /// Check if process is in unexpected location
    fn is_unexpected_location(&self, process: &ProcessInfo) -> bool {
        let expected_paths = [
            "/usr/bin/",
            "/usr/sbin/",
            "/bin/",
            "/sbin/",
            "/usr/lib/",
            "/lib/",
            "/opt/",
            "/usr/local/bin/",
        ];

        let path = process.name.to_lowercase();
        !expected_paths
            .iter()
            .any(|&expected| path.starts_with(expected))
    }

    /// Check if process runs with high privileges
    fn is_high_privilege(&self, process: &ProcessInfo) -> bool {
        // This would need to be implemented with actual user/group checking
        // For now, we'll use a simple heuristic
        process.name.contains("root") || process.name.contains("sudo")
    }

    /// Check if process has network exposure
    fn has_network_exposure(&self, _process: &ProcessInfo) -> bool {
        // Check if process is listening on all interfaces (0.0.0.0)
        // This would need to be implemented with actual network interface checking
        false // Placeholder
    }

    /// Check for process anomalies
    fn has_process_anomaly(&self, _process: &ProcessInfo) -> bool {
        // Check for unusual process characteristics
        // This could include checking for:
        // - Unusual memory usage
        // - Unusual CPU usage
        // - Unusual file descriptors
        // - Unusual network connections
        false // Placeholder
    }

    /// Escalate risk level
    fn escalate_risk(&self, current: RiskLevel) -> RiskLevel {
        match current {
            RiskLevel::Low => RiskLevel::Medium,
            RiskLevel::Medium => RiskLevel::High,
            RiskLevel::High => RiskLevel::Critical,
            RiskLevel::Critical => RiskLevel::Critical,
        }
    }

    /// Classify service type
    fn classify_service_type(&self, process: &ProcessInfo) -> ServiceType {
        let name = process.name.to_lowercase();

        if name.contains("nginx") || name.contains("apache") || name.contains("httpd") {
            ServiceType::WebServer
        } else if name.contains("mysql") || name.contains("postgres") || name.contains("redis") {
            ServiceType::Database
        } else if name.contains("ssh") || name.contains("sshd") {
            ServiceType::SSH
        } else if name.contains("postfix") || name.contains("sendmail") {
            ServiceType::Mail
        } else if name.contains("bind") || name.contains("named") {
            ServiceType::DNS
        } else {
            ServiceType::Custom
        }
    }

    /// Get expected location for process
    fn get_expected_location(&self, process: &ProcessInfo) -> String {
        match self.classify_service_type(process) {
            ServiceType::WebServer => "/usr/sbin/".to_string(),
            ServiceType::Database => "/usr/bin/".to_string(),
            ServiceType::SSH => "/usr/sbin/".to_string(),
            ServiceType::Mail => "/usr/sbin/".to_string(),
            ServiceType::DNS => "/usr/sbin/".to_string(),
            ServiceType::Custom => "/usr/local/bin/".to_string(),
        }
    }

    /// Calculate binary hash
    async fn calculate_binary_hash(&self, process: &ProcessInfo) -> Result<String> {
        // This would calculate SHA256 hash of the binary
        // For now, return a placeholder
        Ok(format!("hash_{}", process.pid))
    }

    /// Get parent process
    async fn get_parent_process(&self, _process: &ProcessInfo) -> Result<String> {
        // This would get the actual parent process
        // For now, return a placeholder
        Ok("unknown".to_string())
    }

    /// Get network interface
    fn get_network_interface(&self, _process: &ProcessInfo) -> String {
        // This would get the actual network interface
        // For now, return a placeholder
        "0.0.0.0".to_string()
    }

    /// Generate security recommendations
    fn generate_recommendations(
        &self,
        suspicious_processes: &[SuspiciousProcess],
    ) -> Vec<SecurityRecommendation> {
        let mut recommendations = Vec::new();

        if !suspicious_processes.is_empty() {
            recommendations.push(SecurityRecommendation {
                title: "Investigate Suspicious Processes".to_string(),
                description: format!(
                    "{} suspicious processes detected",
                    suspicious_processes.len()
                ),
                action: "Review and terminate suspicious processes immediately".to_string(),
                priority: RiskLevel::High,
                affected_processes: suspicious_processes.iter().map(|p| p.port).collect(),
            });
        }

        recommendations
    }

    /// Calculate security score
    fn calculate_security_score(
        &self,
        suspicious_processes: &[SuspiciousProcess],
        total_processes: usize,
    ) -> f64 {
        if total_processes == 0 {
            return 100.0;
        }

        let suspicious_count = suspicious_processes.len();
        let base_score = 100.0 - (suspicious_count as f64 / total_processes as f64) * 100.0;

        // Penalize based on risk levels
        let risk_penalty = suspicious_processes
            .iter()
            .map(|p| match p.risk_level {
                RiskLevel::Low => 5.0,
                RiskLevel::Medium => 10.0,
                RiskLevel::High => 20.0,
                RiskLevel::Critical => 30.0,
            })
            .sum::<f64>();

        (base_score - risk_penalty).max(0.0)
    }

    /// Compare with baseline file
    async fn compare_with_baseline(
        &self,
        baseline_path: &str,
        current_processes: &HashMap<u16, ProcessInfo>,
    ) -> Result<BaselineComparison> {
        if !Path::new(baseline_path).exists() {
            return Err(anyhow::anyhow!(
                "Baseline file not found: {}",
                baseline_path
            ));
        }

        let baseline_data = fs::read_to_string(baseline_path)?;
        let baseline_processes: HashMap<u16, ProcessInfo> = serde_json::from_str(&baseline_data)?;

        let mut new_processes = Vec::new();
        let mut removed_processes = Vec::new();
        let mut changed_processes = Vec::new();

        // Find new processes
        for (port, process) in current_processes {
            if !baseline_processes.contains_key(port) {
                new_processes.push(process.clone());
            }
        }

        // Find removed processes
        for (port, process) in &baseline_processes {
            if !current_processes.contains_key(port) {
                removed_processes.push(process.clone());
            }
        }

        // Find changed processes
        for (port, current_process) in current_processes {
            if let Some(baseline_process) = baseline_processes.get(port) {
                if current_process != baseline_process {
                    changed_processes.push(ProcessChange {
                        port: *port,
                        old_process: baseline_process.clone(),
                        new_process: current_process.clone(),
                        change_type: ProcessChangeType::BinaryChanged, // Simplified
                    });
                }
            }
        }

        Ok(BaselineComparison {
            baseline_file: baseline_path.to_string(),
            new_processes,
            removed_processes,
            changed_processes,
        })
    }
}
