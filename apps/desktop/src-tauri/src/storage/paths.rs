use std::path::PathBuf;

use tauri::{AppHandle, Manager};

pub fn app_data_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Không lấy được app_data_dir: {e}"))
}

pub fn job_workspace_dir(app_handle: &AppHandle, job_id: &str) -> Result<PathBuf, String> {
    let mut base = app_data_dir(app_handle)?;
    base.push("SiroMix");
    base.push("jobs");
    base.push(job_id);
    Ok(base)
}
