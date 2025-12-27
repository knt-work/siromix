mod storage;

use serde::{Deserialize, Serialize};
use std::path::Path;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Deserialize)]
pub struct AnalyzeDocxPayload {
    #[serde(rename = "jobId")]
    pub job_id: String,
    #[serde(rename = "sourcePath")]
    pub source_path: String,
}

#[derive(Serialize)]
pub struct AnalyzeDocxResponse {
    pub ok: bool,
    #[serde(rename = "jobId")]
    pub job_id: String,
    #[serde(rename = "workspacePath")]
    pub workspace_path: String,
}

#[tauri::command]
fn analyze_docx(
    app_handle: tauri::AppHandle,
    payload: AnalyzeDocxPayload,
) -> Result<AnalyzeDocxResponse, String> {
    use crate::storage::{fs, paths};

    let workspace_dir =
        paths::job_workspace_dir(&app_handle, &payload.job_id)?;

    fs::ensure_dir(&workspace_dir)?;

    let source = Path::new(&payload.source_path);
    let destination = workspace_dir.join("source.docx");

    fs::copy_file(source, &destination)?;

    let workspace_path_str = workspace_dir
        .to_str()
        .ok_or_else(|| {
            "Không chuyển được đường dẫn workspace thành chuỗi UTF-8".to_string()
        })?
        .to_string();

    Ok(AnalyzeDocxResponse {
        ok: true,
        job_id: payload.job_id,
        workspace_path: workspace_path_str,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, analyze_docx])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
