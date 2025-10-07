use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JsPmReport {
    pub npm: bool,
    pub pnpm: bool,
    pub yarn: bool,
}

pub async fn scan_js_pm() -> JsPmReport {
    JsPmReport { npm: false, pnpm: false, yarn: false }
}

