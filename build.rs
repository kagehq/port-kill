fn main() {
    #[cfg(target_os = "windows")]
    {
        if std::path::Path::new("assets/port-kill.ico").exists() {
            let res_build = (|| -> Result<(), Box<dyn std::error::Error>> {
                let mut res = winres::WindowsResource::new();
                res.set_icon("assets/port-kill.ico");
                res.compile()?;
                Ok(())
            })();
            if let Err(e) = res_build {
                println!("cargo:warning=winres failed: {}", e);
            }
        }
    }
}


