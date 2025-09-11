fn main() {
    #[cfg(all(target_os = "windows", feature = "embed_icon"))]
    {
        embed_icon();
    }
}

#[cfg(all(target_os = "windows", feature = "embed_icon"))]
fn embed_icon() {
    if std::path::Path::new("app_icon.rc").exists() && std::path::Path::new("assets/port-kill.ico").exists() {
        embed_resource::compile("app_icon.rc");
    } else {
        println!("cargo:warning=Icon files not found, skipping resource embedding");
    }
}


