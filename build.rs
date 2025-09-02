fn main() {
    #[cfg(target_os = "windows")]
    {
        // Only run winres when available (on Windows builds)
        if std::path::Path::new("assets/port-kill.ico").exists() {
            if let Ok(mut res) = try_winres() {
                res.set_icon("assets/port-kill.ico");
                let _ = res.compile();
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn try_winres() -> Result<winres::WindowsResource, ()> {
    Ok(winres::WindowsResource::new())
}

#[cfg(not(target_os = "windows"))]
fn try_winres() -> Result<(), ()> { Err(()) }


