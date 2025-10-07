use super::types::{CleanResponse, CleanSummary};
use super::detect::{detect_rust_caches, detect_js_caches, detect_npx_caches, detect_js_pm_caches, detect_python_caches, detect_java_caches, detect_hf_caches, detect_torch_caches, detect_vercel_caches, detect_cloudflare_caches};
use super::backup::safe_delete_entries;
use std::path::Path;

pub async fn clean_caches(lang: &str, include_npx: bool, include_js_pm: bool, safe_delete: bool, _force: bool, include_hf: bool, include_torch: bool, include_vercel: bool, include_cloudflare: bool, stale_days: Option<u32>) -> CleanResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    let mut entries = Vec::new();
    
    // If specific flags are provided, only use those
    if include_npx || include_js_pm || include_hf || include_torch || include_vercel || include_cloudflare {
        // NPX caches
        if include_npx {
            entries.extend(detect_npx_caches(stale_days));
        }
        
        // JS Package Manager caches
        if include_js_pm {
            entries.extend(detect_js_pm_caches());
        }
        
        // Specialized integrations
        if include_hf {
            entries.extend(detect_hf_caches());
        }
        
        if include_torch {
            entries.extend(detect_torch_caches());
        }
        
        if include_vercel {
            entries.extend(detect_vercel_caches());
        }
        
        if include_cloudflare {
            entries.extend(detect_cloudflare_caches());
        }
    } else {
        // Use language-based detection
        // Rust caches
        if lang == "auto" || lang == "rust" {
            entries.extend(detect_rust_caches(Path::new(&cwd)));
        }
        
        // JavaScript/TypeScript caches
        if lang == "auto" || lang == "js" {
            entries.extend(detect_js_caches(Path::new(&cwd)));
        }
        
        // Python caches
        if lang == "auto" || lang == "py" {
            entries.extend(detect_python_caches());
        }
        
        // Java caches
        if lang == "auto" || lang == "java" {
            entries.extend(detect_java_caches());
        }
    }

    match safe_delete_entries(&entries, safe_delete).await {
        Ok((deleted, backup_path)) => {
            let freed_bytes: u64 = deleted.iter().map(|e| e.size_bytes).sum();
            let deleted_count = deleted.len();
            CleanResponse {
                deleted,
                backed_up_to: backup_path,
                summary: CleanSummary { 
                    freed_bytes: freed_bytes, 
                    deleted_count: deleted_count 
                },
            }
        }
        Err(e) => {
            eprintln!("Error during cleanup: {}", e);
            CleanResponse {
                deleted: vec![],
                backed_up_to: None,
                summary: CleanSummary { freed_bytes: 0, deleted_count: 0 },
            }
        }
    }
}

