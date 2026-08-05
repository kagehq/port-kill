//! Windows autostart management via the HKCU Run registry key.
//! User-scoped (no admin required) and fully reversible from the tray menu.

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "PortKill";

// Cache the registry state so menu rebuilds don't spawn reg.exe every time.
static ENABLED: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Whether the app is registered to start with Windows.
pub fn is_enabled() -> bool {
    if !INITIALIZED.load(Ordering::Acquire) {
        ENABLED.store(query_registry(), Ordering::Release);
        INITIALIZED.store(true, Ordering::Release);
    }
    ENABLED.load(Ordering::Acquire)
}

fn query_registry() -> bool {
    crate::hidden_command::hidden_command("reg")
        .args(["query", RUN_KEY, "/v", VALUE_NAME])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Toggle autostart. When enabling, the current executable and command-line
/// arguments are stored so the app restarts with the same monitoring options.
/// Returns the new state.
pub fn toggle() -> Result<bool> {
    if is_enabled() {
        let output = crate::hidden_command::hidden_command("reg")
            .args(["delete", RUN_KEY, "/v", VALUE_NAME, "/f"])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "reg delete failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        ENABLED.store(false, Ordering::Release);
        Ok(false)
    } else {
        let exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args()
            .skip(1)
            .map(|arg| {
                if arg.contains(' ') {
                    format!("\"{}\"", arg)
                } else {
                    arg
                }
            })
            .collect();
        let command = if args.is_empty() {
            format!("\"{}\"", exe.display())
        } else {
            format!("\"{}\" {}", exe.display(), args.join(" "))
        };
        let output = crate::hidden_command::hidden_command("reg")
            .args([
                "add", RUN_KEY, "/v", VALUE_NAME, "/t", "REG_SZ", "/d", &command, "/f",
            ])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "reg add failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        ENABLED.store(true, Ordering::Release);
        Ok(true)
    }
}
