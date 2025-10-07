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
