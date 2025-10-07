use super::types::RestoreResponse;
use super::backup::{find_latest_backup, restore_from_backup};

pub async fn restore_last_backup() -> RestoreResponse {
    match find_latest_backup() {
        Ok(Some(backup_path)) => {
            match restore_from_backup(&backup_path).await {
                Ok(count) => RestoreResponse {
                    restored_from: backup_path.to_string_lossy().to_string(),
                    restored_count: count,
                },
                Err(e) => {
                    eprintln!("Error restoring backup: {}", e);
                    RestoreResponse {
                        restored_from: String::new(),
                        restored_count: 0,
                    }
                }
            }
        }
        Ok(None) => {
            eprintln!("No backup found to restore");
            RestoreResponse {
                restored_from: String::new(),
                restored_count: 0,
            }
        }
        Err(e) => {
            eprintln!("Error finding backup: {}", e);
            RestoreResponse {
                restored_from: String::new(),
                restored_count: 0,
            }
        }
    }
}

