use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

const GITHUB_API_URL: &str = "https://api.github.com/repos/kagehq/port-kill/releases/latest";
const CHECK_INTERVAL_DAYS: u64 = 1; // Check for updates once per day

#[derive(Debug, Serialize, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    published_at: String,
    html_url: String,
    body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    current_version: String,
    latest_version: String,
    is_update_available: bool,
    release_url: String,
    release_notes: String,
    last_checked: u64,
}

pub fn check_for_updates(current_version: &str) -> Result<Option<UpdateInfo>> {
    // Check if we should skip the update check (too recent)
    if should_skip_check()? {
        return Ok(None);
    }

    // Fetch latest release from GitHub
    let latest_release = fetch_latest_release()?;
    let latest_version = latest_release.tag_name.trim_start_matches('v');
    
    // Compare versions
    let is_update_available = compare_versions(current_version, latest_version);
    
    // Update last check time
    update_last_check_time()?;
    
    if is_update_available {
        Ok(Some(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_version.to_string(),
            is_update_available: true,
            release_url: latest_release.html_url,
            release_notes: latest_release.body,
            last_checked: current_timestamp(),
        }))
    } else {
        Ok(None)
    }
}

fn fetch_latest_release() -> Result<GitHubRelease> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(GITHUB_API_URL)
        .header("User-Agent", "port-kill-update-checker")
        .send()?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch release info: {}", response.status()));
    }
    
    let release: GitHubRelease = response.json()?;
    Ok(release)
}

fn compare_versions(current: &str, latest: &str) -> bool {
    // Simple version comparison (assumes semantic versioning)
    // This is a basic implementation - could be enhanced with proper semver parsing
    current != latest
}

fn should_skip_check() -> Result<bool> {
    let last_check = get_last_check_time()?;
    let now = current_timestamp();
    let days_since_check = (now - last_check) / (24 * 60 * 60);
    
    Ok(days_since_check < CHECK_INTERVAL_DAYS)
}

fn get_last_check_time() -> Result<u64> {
    let cache_dir = get_cache_dir()?;
    let check_file = cache_dir.join("last_update_check");
    
    if check_file.exists() {
        let content = std::fs::read_to_string(&check_file)?;
        Ok(content.trim().parse().unwrap_or(0))
    } else {
        Ok(0)
    }
}

fn update_last_check_time() -> Result<()> {
    let cache_dir = get_cache_dir()?;
    std::fs::create_dir_all(&cache_dir)?;
    
    let check_file = cache_dir.join("last_update_check");
    std::fs::write(check_file, current_timestamp().to_string())?;
    
    Ok(())
}

fn get_cache_dir() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let cache_dir = std::path::PathBuf::from(home).join(".port-kill");
    std::fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn self_update() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    
    // Check for updates
    let update_info = match check_for_updates(current_version)? {
        Some(info) => info,
        None => {
            println!("✅ You're already running the latest version ({})", current_version);
            return Ok(());
        }
    };
    
    println!("🔄 Updating from {} to {}...", update_info.current_version, update_info.latest_version);
    
    // Get the current executable path
    let current_exe = std::env::current_exe()?;
    let _current_exe_path = current_exe.to_string_lossy().to_string();
    
    // Determine platform-specific download URL
    let download_url = get_platform_download_url()?;
    
    // Download the new binary
    println!("📥 Downloading latest version...");
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&download_url)
        .header(reqwest::header::USER_AGENT, "port-kill-updater")
        .send()?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to download update: HTTP {}", response.status()));
    }
    
    let new_binary = response.bytes()?;
    
    // Create a temporary file for the new binary
    let temp_dir = std::env::temp_dir();
    let temp_exe = temp_dir.join("port-kill-new.exe");
    std::fs::write(&temp_exe, new_binary)?;
    
    // Make the new binary executable (Unix systems)
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_exe)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_exe, perms)?;
    }
    
    // Replace the current binary
    println!("🔄 Replacing current binary...");
    
    // On Windows, we need to use a different approach due to file locking
    #[cfg(target_os = "windows")]
    {
        // Create a batch script to replace the binary after the current process exits
        let batch_content = format!(
            r#"@echo off
timeout /t 2 /nobreak >nul
move "{}" "{}"
del "%~f0"
"#,
            temp_exe.to_string_lossy(),
            current_exe_path
        );
        
        let batch_file = temp_dir.join("port-kill-update.bat");
        std::fs::write(&batch_file, batch_content)?;
        
        // Execute the batch file
        std::process::Command::new("cmd")
            .args(&["/c", "start", "/b", &batch_file.to_string_lossy()])
            .spawn()?;
        
        println!("✅ Update will complete after you restart the application.");
        println!("🔗 Release notes: {}", update_info.release_url);
        return Ok(());
    }
    
    // On Unix systems, we can replace directly
    #[cfg(not(target_os = "windows"))]
    {
        std::fs::copy(&temp_exe, &current_exe)?;
        std::fs::remove_file(&temp_exe)?;
    }
    
    println!("✅ Update completed successfully!");
    println!("🔗 Release notes: {}", update_info.release_url);
    println!("💡 Restart the application to use the new version.");
    
    Ok(())
}

fn get_platform_download_url() -> Result<String> {
    let _platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    
    // Get the latest release info to construct the download URL
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(GITHUB_API_URL)
        .header(reqwest::header::USER_AGENT, "port-kill-updater")
        .send()?;
    
    let release: GitHubRelease = response.json()?;
    let tag_name = release.tag_name;
    
    let binary_name = if cfg!(target_os = "windows") {
        "port-kill-windows.exe"
    } else if cfg!(target_os = "macos") {
        "port-kill-macos"
    } else {
        "port-kill-linux"
    };
    
    Ok(format!(
        "https://github.com/kagehq/port-kill/releases/download/{}/{}",
        tag_name, binary_name
    ))
}

pub fn print_update_notification(update_info: &UpdateInfo) {
    println!();
    println!("🔄 Update Available!");
    println!("==================");
    println!("Current version: {}", update_info.current_version);
    println!("Latest version:  {}", update_info.latest_version);
    println!();
    println!("📥 To update:");
    println!("   curl -fsSL https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.sh | bash");
    println!();
    println!("🔗 Release notes: {}", update_info.release_url);
    println!();
}

pub fn print_update_check_result(update_info: &UpdateInfo) {
    if update_info.is_update_available {
        print_update_notification(update_info);
    } else {
        println!("✅ You're running the latest version ({})", update_info.current_version);
    }
}
