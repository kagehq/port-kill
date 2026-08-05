//! Spawn external commands without flashing a console window.
//!
//! On Windows, `port-kill` is built with the GUI subsystem, so it has no
//! console of its own. Every console-subsystem child (netstat, tasklist,
//! taskkill, reg, ...) would otherwise allocate a brand-new *visible*
//! console window, flashing on screen on every scan.
//!
//! All of these children are short-lived query commands whose output is
//! captured through pipes, so they never need a console at all.

use std::ffi::OsStr;
use std::process::Command;

/// Build a `Command` that won't open a console window on Windows.
/// On other platforms this is identical to `Command::new`.
pub fn hidden_command<S: AsRef<OsStr>>(program: S) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
