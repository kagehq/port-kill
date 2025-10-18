use super::types::CacheEntry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupManifest {
    pub timestamp: DateTime<Utc>,
    pub entries: Vec<CacheEntry>,
    pub backup_dir: String,
}

pub fn get_backup_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".cachekill-backup")
}

pub fn create_backup_dir() -> Result<PathBuf, std::io::Error> {
    let backup_dir = get_backup_dir();
    fs::create_dir_all(&backup_dir)?;
    Ok(backup_dir)
}

pub fn get_timestamped_dir() -> PathBuf {
    let backup_dir = get_backup_dir();
    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%SZ");
    backup_dir.join(timestamp.to_string())
}

pub async fn safe_delete_entries(
    entries: &[CacheEntry],
    safe_delete: bool,
) -> Result<(Vec<CacheEntry>, Option<String>), std::io::Error> {
    let mut deleted = Vec::new();
    let mut backup_path = None;

    if safe_delete {
        let timestamped_dir = get_timestamped_dir();
        fs::create_dir_all(&timestamped_dir)?;
        backup_path = Some(timestamped_dir.to_string_lossy().to_string());

        // Create manifest
        let manifest = BackupManifest {
            timestamp: Utc::now(),
            entries: entries.to_vec(),
            backup_dir: timestamped_dir.to_string_lossy().to_string(),
        };

        let manifest_path = timestamped_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json)?;

        // Move entries to backup
        for entry in entries {
            let src = Path::new(&entry.path);
            if src.exists() {
                let dst = timestamped_dir.join(src.file_name().unwrap_or_default());
                if let Err(e) = fs::rename(src, &dst) {
                    eprintln!("Warning: Failed to backup {}: {}", entry.path, e);
                } else {
                    deleted.push(entry.clone());
                }
            }
        }
    } else {
        // Direct deletion without backup
        for entry in entries {
            let path = Path::new(&entry.path);
            if path.exists() {
                if path.is_dir() {
                    if let Err(e) = fs::remove_dir_all(path) {
                        eprintln!("Warning: Failed to delete {}: {}", entry.path, e);
                    } else {
                        deleted.push(entry.clone());
                    }
                } else if path.is_file() {
                    if let Err(e) = fs::remove_file(path) {
                        eprintln!("Warning: Failed to delete {}: {}", entry.path, e);
                    } else {
                        deleted.push(entry.clone());
                    }
                }
            }
        }
    }

    Ok((deleted, backup_path))
}

pub fn find_latest_backup() -> Result<Option<PathBuf>, std::io::Error> {
    let backup_dir = get_backup_dir();
    if !backup_dir.exists() {
        return Ok(None);
    }

    let mut entries: Vec<_> = fs::read_dir(&backup_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    entries.sort_by_key(|e| {
        e.metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::UNIX_EPOCH)
    });

    Ok(entries.last().map(|e| e.path()))
}

pub async fn restore_from_backup(backup_path: &Path) -> Result<usize, std::io::Error> {
    let manifest_path = backup_path.join("manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: BackupManifest = serde_json::from_str(&manifest_content)?;

    let mut restored_count = 0;

    for entry in &manifest.entries {
        let backup_file = backup_path.join(Path::new(&entry.path).file_name().unwrap_or_default());
        let original_path = Path::new(&entry.path);

        if backup_file.exists() {
            // Ensure parent directory exists
            if let Some(parent) = original_path.parent() {
                fs::create_dir_all(parent)?;
            }

            if let Err(e) = fs::rename(&backup_file, original_path) {
                eprintln!("Warning: Failed to restore {}: {}", entry.path, e);
            } else {
                restored_count += 1;
            }
        }
    }

    // Clean up backup directory after successful restore
    if restored_count > 0 {
        let _ = fs::remove_dir_all(backup_path);
    }

    Ok(restored_count)
}
