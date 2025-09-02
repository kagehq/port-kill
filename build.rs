#[cfg(target_os = "windows")]
fn main() {
    // Embed an icon if available
    let mut res = winres::WindowsResource::new();
    // Prefer repo icon path
    if std::path::Path::new("assets/port-kill.ico").exists() {
        res.set_icon("assets/port-kill.ico");
    }
    // Build the resources; ignore errors if the toolchain is missing
    let _ = res.compile();
}

#[cfg(not(target_os = "windows"))]
fn main() {}


