use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub notes: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub async fn doctor() -> DoctorReport {
    let mut notes = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Check system environment
    if let Ok(home) = std::env::var("HOME") {
        notes.push(format!("Home directory: {}", home));

        // Check for common cache directories
        let cache_dirs = [
            (".cargo", "Rust toolchain cache"),
            (".npm", "npm cache"),
            (".pnpm-store", "pnpm store"),
            (".yarn/cache", "yarn cache"),
            (".cache/huggingface", "Hugging Face cache"),
            (".cache/torch", "PyTorch cache"),
            (".vercel", "Vercel cache"),
            (".cloudflare", "Cloudflare cache"),
            (".m2", "Maven cache"),
        ];

        for (dir, description) in &cache_dirs {
            let path = PathBuf::from(&home).join(dir);
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.is_dir() {
                        notes.push(format!("Found {}: {}", description, path.display()));
                    }
                }
            }
        }

        // Check disk space (Unix only)
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(statvfs) = nix::sys::statvfs::statvfs(home.as_str()) {
                let total_space = (statvfs.blocks() as u64) * (statvfs.fragment_size() as u64);
                let free_space =
                    (statvfs.blocks_available() as u64) * (statvfs.fragment_size() as u64);
                let used_percent = ((total_space - free_space) as f64 / total_space as f64) * 100.0;

                notes.push(format!("Disk usage: {:.1}% used", used_percent));

                if used_percent > 90.0 {
                    warnings.push("Disk usage is above 90% - consider cleaning caches".to_string());
                } else if used_percent > 80.0 {
                    warnings.push("Disk usage is above 80% - monitor cache sizes".to_string());
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            notes.push("Disk usage: Check manually on Windows".to_string());
        }

        // Check for large cache directories
        let large_caches = [
            (".cargo", 1_000_000_000),      // 1GB
            (".npm", 500_000_000),          // 500MB
            (".pnpm-store", 1_000_000_000), // 1GB
        ];

        for (dir, threshold) in &large_caches {
            let path = PathBuf::from(&home).join(dir);
            if path.exists() {
                if let Ok(size) = get_dir_size(&path) {
                    if size > *threshold {
                        warnings.push(format!(
                            "Large cache detected: {} ({:.1} MB)",
                            dir,
                            size as f64 / 1_000_000.0
                        ));
                    }
                }
            }
        }
    } else {
        errors.push("HOME environment variable not set".to_string());
    }

    // Check for backup directories
    let backup_dir = PathBuf::from(".cachekill-backup");
    if backup_dir.exists() {
        if let Ok(entries) = fs::read_dir(&backup_dir) {
            let count = entries.count();
            notes.push(format!(
                "Found {} backup(s) from previous clean operations",
                count
            ));
        }
    }

    // Check current working directory for cache files
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cache_patterns = ["target", "node_modules", "__pycache__", ".gradle", "build"];

    for pattern in &cache_patterns {
        let path = cwd.join(pattern);
        if path.exists() {
            notes.push(format!("Found {} in current directory", pattern));
        }
    }

    let ok = errors.is_empty();
    DoctorReport {
        ok,
        notes,
        warnings,
        errors,
    }
}

fn get_dir_size(path: &std::path::Path) -> Result<u64, std::io::Error> {
    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(path) {
        if let Ok(entry) = entry {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                }
            }
        }
    }
    Ok(total)
}
