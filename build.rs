fn main() {
    #[cfg(all(target_os = "windows", feature = "embed_icon"))]
    {
        embed_icon();
    }
}

#[cfg(all(target_os = "windows", feature = "embed_icon"))]
fn embed_icon() {
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


