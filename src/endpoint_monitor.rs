use crate::cli::Args;
use crate::process_monitor::ProcessMonitor;
use crate::security_audit::SecurityAuditor;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{interval, sleep};

/// Data structure for endpoint monitoring payload
#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointPayload {
    pub timestamp: DateTime<Utc>,
    pub server: String,
    pub environment: String,
    pub team: String,
    pub ports: Vec<PortStatus>,
    pub security_audit: Option<SecurityAuditData>,
    pub summary: PortSummary,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortStatus {
    pub port: u16,
    pub status: String, // "occupied" or "free"
    pub process: Option<String>,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
    pub container: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityAuditData {
    pub suspicious_ports: Vec<u16>,
    pub risk_score: f64,
    pub unauthorized_processes: Vec<String>,
    pub baseline_violations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortSummary {
    pub total_ports: usize,
    pub occupied_ports: usize,
    pub free_ports: usize,
    pub suspicious_ports: usize,
}

/// Endpoint monitor for sending periodic data to external endpoints
pub struct EndpointMonitor {
    client: Client,
    endpoint_url: String,
    auth_header: Option<String>,
    custom_fields: HashMap<String, String>,
    include_audit: bool,
    retries: u32,
    _timeout: Duration,
    process_monitor: ProcessMonitor,
    security_auditor: Option<SecurityAuditor>,
}

impl EndpointMonitor {
    /// Create a new endpoint monitor
    pub fn new(args: &Args) -> Result<Self> {
        let endpoint_url = args
            .monitor_endpoint
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Endpoint URL is required for monitoring"))?
            .clone();

        // Create HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(args.endpoint_timeout))
            .build()
            .context("Failed to create HTTP client")?;

        // Parse authentication header
        let auth_header = if let Some(auth) = &args.endpoint_auth {
            Some(format!("Authorization: {}", auth))
        } else {
            None
        };

        // Parse custom fields
        let custom_fields = if let Some(fields) = &args.endpoint_fields {
            let mut map = HashMap::new();
            for field in fields {
                if let Some((key, value)) = field.split_once('=') {
                    map.insert(key.to_string(), value.to_string());
                }
            }
            map
        } else {
            HashMap::new()
        };

        // Create process monitor
        let (update_sender, _update_receiver) = crossbeam_channel::bounded(100);
        let ports_to_scan = args.get_ports_to_monitor();
        let process_monitor = ProcessMonitor::new_with_performance(
            update_sender,
            ports_to_scan,
            args.docker,
            args.verbose,
            None,
            args.performance,
        )?;

        // Create security auditor if audit is enabled
        let security_auditor = if args.endpoint_include_audit {
            // Parse suspicious ports from comma-separated string
            let suspicious_ports: Vec<u16> = args
                .suspicious_ports
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            Some(SecurityAuditor::new(
                suspicious_ports,
                args.baseline_file.clone(),
                args.suspicious_only,
            ))
        } else {
            None
        };

        Ok(Self {
            client,
            endpoint_url,
            auth_header,
            custom_fields,
            include_audit: args.endpoint_include_audit,
            retries: args.endpoint_retries,
            _timeout: Duration::from_secs(args.endpoint_timeout),
            process_monitor,
            security_auditor,
        })
    }

    /// Run the endpoint monitor with dual intervals
    pub async fn run(&mut self, args: &Args) -> Result<()> {
        let scan_interval = Duration::from_secs(args.scan_interval);
        let send_interval = Duration::from_secs(args.send_interval);

        let mut scan_timer = interval(scan_interval);
        let mut send_timer = interval(send_interval);

        log::info!("Starting endpoint monitor:");
        log::info!("  - Endpoint: {}", self.endpoint_url);
        log::info!("  - Scan interval: {}s", args.scan_interval);
        log::info!("  - Send interval: {}s", args.send_interval);
        log::info!("  - Include audit: {}", self.include_audit);

        loop {
            tokio::select! {
                _ = scan_timer.tick() => {
                    // Scan processes at high frequency
                    if let Err(e) = self.scan_processes().await {
                        log::warn!("Failed to scan processes: {}", e);
                    }
                }
                _ = send_timer.tick() => {
                    // Send data to endpoint at lower frequency
                    if let Err(e) = self.send_to_endpoint().await {
                        log::warn!("Failed to send data to endpoint: {}", e);
                    }
                }
            }
        }
    }

    /// Scan processes and update internal state
    async fn scan_processes(&mut self) -> Result<()> {
        self.process_monitor.scan_processes().await?;
        Ok(())
    }

    /// Send data to the configured endpoint
    async fn send_to_endpoint(&self) -> Result<()> {
        let payload = self.build_payload().await?;

        for attempt in 1..=self.retries {
            match self.send_payload(&payload).await {
                Ok(_) => {
                    log::debug!("Successfully sent data to endpoint (attempt {})", attempt);
                    return Ok(());
                }
                Err(e) => {
                    log::warn!(
                        "Failed to send data to endpoint (attempt {}): {}",
                        attempt,
                        e
                    );
                    if attempt < self.retries {
                        let delay = Duration::from_secs(attempt as u64 * 2); // Exponential backoff
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to send data to endpoint after {} attempts",
            self.retries
        ))
    }

    /// Send payload to endpoint with retry logic
    async fn send_payload(&self, payload: &EndpointPayload) -> Result<()> {
        let mut request = self.client.post(&self.endpoint_url).json(payload);

        // Add authentication header if provided
        if let Some(auth) = &self.auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Endpoint returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        Ok(())
    }

    /// Build the payload to send to the endpoint
    async fn build_payload(&self) -> Result<EndpointPayload> {
        let processes = self.process_monitor.get_processes();
        let ports_to_monitor = self.process_monitor.get_ports_to_monitor();

        // Build port status list
        let mut ports = Vec::new();
        for &port in ports_to_monitor {
            if let Some((_, process)) = processes.iter().find(|(p, _)| **p == port) {
                ports.push(PortStatus {
                    port,
                    status: "occupied".to_string(),
                    process: Some(process.name.clone()),
                    pid: Some(process.pid as u32),
                    uptime: None, // ProcessInfo doesn't have uptime field
                    container: process.container_name.clone(),
                });
            } else {
                ports.push(PortStatus {
                    port,
                    status: "free".to_string(),
                    process: None,
                    pid: None,
                    uptime: None,
                    container: None,
                });
            }
        }

        // Calculate summary
        let occupied_ports = ports.iter().filter(|p| p.status == "occupied").count();
        let free_ports = ports.len() - occupied_ports;

        // Get security audit data if enabled
        let security_audit = if self.include_audit {
            if let Some(ref auditor) = self.security_auditor {
                let audit_result = auditor.perform_audit(processes.clone()).await?;
                Some(SecurityAuditData {
                    suspicious_ports: audit_result
                        .suspicious_processes
                        .iter()
                        .map(|p| p.port)
                        .collect(),
                    risk_score: audit_result.security_score,
                    unauthorized_processes: audit_result
                        .suspicious_processes
                        .iter()
                        .map(|p| p.process_info.name.clone())
                        .collect(),
                    baseline_violations: audit_result
                        .baseline_comparison
                        .as_ref()
                        .map_or_else(Vec::new, |bc| {
                            bc.new_processes.iter().map(|p| p.name.clone()).collect()
                        }),
                })
            } else {
                None
            }
        } else {
            None
        };

        // Count suspicious ports
        let suspicious_ports = if let Some(ref audit) = security_audit {
            audit.suspicious_ports.len()
        } else {
            0
        };

        // Get server information
        let server = self
            .custom_fields
            .get("server")
            .cloned()
            .unwrap_or_else(|| {
                hostname::get()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });

        let environment = self
            .custom_fields
            .get("environment")
            .cloned()
            .unwrap_or_else(|| "development".to_string());

        let team = self
            .custom_fields
            .get("team")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let total_ports = ports.len();

        Ok(EndpointPayload {
            timestamp: Utc::now(),
            server,
            environment,
            team,
            ports,
            security_audit,
            summary: PortSummary {
                total_ports,
                occupied_ports,
                free_ports,
                suspicious_ports,
            },
            custom_fields: self.custom_fields.clone(),
        })
    }
}

/// Helper function to get hostname
mod hostname {
    use std::ffi::OsString;

    pub fn get() -> Result<OsString, std::io::Error> {
        std::env::var_os("HOSTNAME")
            .or_else(|| std::env::var_os("COMPUTERNAME"))
            .or_else(|| std::env::var_os("HOST"))
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Could not determine hostname")
            })
    }
}
