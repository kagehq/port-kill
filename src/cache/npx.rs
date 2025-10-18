use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NpxPackageInfo {
    pub name: String,
    pub version: Option<String>,
    pub size_bytes: u64,
    pub last_used_at: Option<String>,
    pub stale: bool,
}

#[derive(Debug, Serialize)]
pub struct NpxReport {
    pub packages: Vec<NpxPackageInfo>,
}

pub async fn analyze_npx(_stale_days: Option<u32>) -> NpxReport {
    NpxReport { packages: vec![] }
}
