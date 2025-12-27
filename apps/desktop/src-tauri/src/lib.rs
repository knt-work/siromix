mod storage;
mod docx;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

use crate::docx::model::ParsedDoc;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<AnalyzeDocxError>>,
}

#[derive(Serialize)]
pub struct AnalyzeDocxError {
    pub code: String,
    #[serde(rename = "questionNumber")]
    pub question_number: u32,
}

#[tauri::command]
fn analyze_docx(
    app_handle: tauri::AppHandle,
    payload: AnalyzeDocxPayload,
) -> Result<AnalyzeDocxResponse, String> {
    use crate::storage::{fs, paths};

    use crate::docx::{assets, parser, read};
    use crate::docx::validator::{self, LabelRunStyle, LabeledOptionRuns};

    let workspace_dir =
        paths::job_workspace_dir(&app_handle, &payload.job_id)?;

    fs::ensure_dir(&workspace_dir)?;

    let source = Path::new(&payload.source_path);
    let destination = workspace_dir.join("source.docx");

    fs::copy_file(source, &destination)?;
    let docx_path = &destination;

    // 1) Read document.xml from the .docx
    let document_xml = read::read_document_xml(docx_path)
        .map_err(|e| format!("Không đọc được document.xml: {:?}", e))?;

    // 2) Extract media into `<workspace>/assets/`
    let assets_dir = workspace_dir.join("assets");
    let _extracted_assets = assets::extract_media(docx_path, &assets_dir)
        .map_err(|e| format!("Không extract media từ docx: {:?}", e))?;

    // 3) Parse -> ParsedDoc (hiện tại mới text-only; math/image sẽ được
    // xử lý chi tiết ở bước parser XML sau).
    let mut parsed_doc = parser::parse_document_xml_to_parsed_doc(&document_xml);

    // 4) Validation: enforce mỗi câu đúng 1 đáp án đúng.
    let mut errors = Vec::new();

    for q in &mut parsed_doc.questions {
        let option_runs: Vec<LabeledOptionRuns> = q
            .options
            .iter()
            .map(|opt| LabeledOptionRuns {
                label: opt.label.clone(),
                // Tạm thời map `locked` sang underline, sẽ được thay bằng
                // styling thật (underline/red) khi parser XML đầy đủ.
                runs: vec![LabelRunStyle {
                    underline: opt.locked,
                    color: None,
                }],
            })
            .collect();

        match validator::detect_correct_label_for_question(q.number, &option_runs) {
            Ok(label) => {
                q.correct_label = label;
            }
            Err(err) => {
                errors.push(AnalyzeDocxError {
                    code: err.code.as_str().to_string(),
                    question_number: err.question_number,
                });
            }
        }
    }

    if !errors.is_empty() {
        return Ok(AnalyzeDocxResponse {
            ok: false,
            job_id: payload.job_id,
            errors: Some(errors),
        });
    }

    // 5) Save `<workspace>/parsed.json` and return { ok: true, jobId }
    let parsed_path = workspace_dir.join("parsed.json");
    let json = serde_json::to_vec_pretty(&parsed_doc)
        .map_err(|e| format!("Không serialize parsed.json: {e}"))?;

    std::fs::write(&parsed_path, json).map_err(|e| {
        format!(
            "Không ghi parsed.json vào {}: {e}",
            parsed_path
                .to_str()
                .unwrap_or("<invalid-path>")
        )
    })?;

    Ok(AnalyzeDocxResponse {
        ok: true,
        job_id: payload.job_id,
        errors: None,
    })
}

/// Đọc `<workspace>/parsed.json` cho một `job_id` và trả về `ParsedDoc` cho frontend.
#[tauri::command]
fn get_parsed(
    app_handle: tauri::AppHandle,
    job_id: String,
) -> Result<ParsedDoc, String> {
    use crate::storage::paths;

    let workspace_dir = paths::job_workspace_dir(&app_handle, &job_id)?;
    let parsed_path = workspace_dir.join("parsed.json");

    if !parsed_path.exists() {
        return Err(format!(
            "Không tìm thấy parsed.json cho job_id {} tại {}",
            job_id,
            parsed_path.to_str().unwrap_or("<invalid-path>")
        ));
    }

    let data = fs::read(&parsed_path).map_err(|e| {
        format!(
            "Không đọc được parsed.json tại {}: {e}",
            parsed_path.to_str().unwrap_or("<invalid-path>")
        )
    })?;

    let parsed: ParsedDoc = serde_json::from_slice(&data)
        .map_err(|e| format!("Không parse được parsed.json: {e}"))?;

    Ok(parsed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, analyze_docx, get_parsed])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
