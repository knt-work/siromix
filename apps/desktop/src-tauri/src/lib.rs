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
async fn analyze_docx(
    app_handle: tauri::AppHandle,
    payload: AnalyzeDocxPayload,
) -> Result<AnalyzeDocxResponse, String> {
    use crate::storage::{fs, paths};

    use crate::docx::{assets, parser, read};
    use crate::docx::validator;

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

    // 2) Extract media into `<workspace>/assets/` (async - uses background tasks for WMF conversion)
    let assets_dir = workspace_dir.join("assets");
    let extracted_assets = assets::extract_media(docx_path, &assets_dir).await
        .map_err(|e| format!("Không extract media từ docx: {:?}", e))?;

    // 3) Parse -> ParsedDoc, đồng thời map các image (kể cả OLE Equation
    // object) theo thứ tự xuất hiện sang danh sách media đã extract.
    let mut parsed_doc = parser::parse_document_xml_to_parsed_doc(
        &document_xml,
        &extracted_assets,
    );

    // 4) Validation: enforce mỗi câu đúng 1 đáp án đúng, dựa trên
    // underline/màu đỏ ở phần label trong document.xml.
    let labeled_option_runs_by_question =
        parser::collect_labeled_option_runs(&document_xml);
    let mut errors = Vec::new();

    for q in &mut parsed_doc.questions {
        if let Some(option_runs) = labeled_option_runs_by_question.get(&q.number) {
            match validator::detect_correct_label_for_question(q.number, option_runs) {
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
        } else {
            // Không tìm thấy bất kỳ label được style cho câu này.
            errors.push(AnalyzeDocxError {
                code: validator::ValidationErrorCode::E020CorrectMarkMissing
                    .as_str()
                    .to_string(),
                question_number: q.number,
            });
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

/// Mix exams - shuffle questions and options to create exam variants
/// This replaces the frontend TypeScript implementation for better performance
#[tauri::command]
fn mix_exams(
    parsed_doc: ParsedDoc,
    num_variants: u32,
    custom_exam_codes: Option<Vec<String>>,
) -> Result<Vec<crate::docx::mixer::MixedExam>, String> {
    use crate::docx::mixer;

    if num_variants == 0 {
        return Err("Number of variants must be greater than 0".to_string());
    }

    if parsed_doc.questions.is_empty() {
        return Err("No questions found in parsed document".to_string());
    }

    // Validate custom exam codes if provided
    if let Some(ref codes) = custom_exam_codes {
        if codes.len() != num_variants as usize {
            return Err(format!(
                "Number of custom exam codes ({}) must match number of variants ({})",
                codes.len(),
                num_variants
            ));
        }
    }

    let variants = mixer::mix_exams(parsed_doc.questions, num_variants as usize, custom_exam_codes);
    Ok(variants)
}

/// Export mixed exams to DOCX and XLSX files
#[tauri::command]
async fn export_mixed_exams(
    app_handle: tauri::AppHandle,
    job_id: String,
    exams: Vec<crate::docx::excel::MixedExam>,
    original_answers: Vec<String>,
    output_dir: String,
) -> Result<ExportResponse, String> {
    use crate::storage::paths;
    use crate::docx::writer::ExamWriter;
    use crate::docx::excel;
    use std::path::PathBuf;

    let workspace_dir = paths::job_workspace_dir(&app_handle, &job_id)?;
    let assets_dir = workspace_dir.join("assets");
    let output_path = PathBuf::from(&output_dir);

    let mut docx_files = Vec::new();

    // Generate DOCX for each exam variant
    for exam in &exams {
        // Convert MixedQuestion to Question format
        let questions: Vec<crate::docx::model::Question> = exam
            .questions
            .iter()
            .map(|mq| {
                // Convert MixedOptions to OptionItems
                let options: Vec<crate::docx::model::OptionItem> = mq.options
                    .iter()
                    .map(|opt| crate::docx::model::OptionItem {
                        label: opt.label.clone(),
                        locked: false, // Options are not locked in mixed exams
                        content: opt.content.clone(),
                    })
                    .collect();

                crate::docx::model::Question {
                    number: mq.display_number as u32,
                    stem: mq.stem.clone(),
                    options,
                    correct_label: mq.correct_answer.clone(),
                }
            })
            .collect();

        let writer = ExamWriter {
            exam_code: exam.exam_code.clone(),
            questions,
            exam_title: "ĐỀ THI GIỮA KỲ I".to_string(),
            subject: "Toán học".to_string(),
            duration_minutes: 90,
            assets_dir: assets_dir.clone(),
        };

        let filename = format!("De_{}.docx", exam.exam_code);
        let file_path = output_path.join(&filename);

        writer
            .write_to_file(&file_path)
            .map_err(|e| format!("Lỗi tạo file {}: {:?}", filename, e))?;

        docx_files.push(filename);
    }

    // Generate XLSX answer key
    let xlsx_filename = "Dap_An.xlsx";
    let xlsx_path = output_path.join(xlsx_filename);

    excel::write_answer_key(&exams, &original_answers, &xlsx_path)
        .map_err(|e| format!("Lỗi tạo file Excel: {:?}", e))?;

    Ok(ExportResponse {
        success: true,
        docx_files,
        xlsx_file: xlsx_filename.to_string(),
        output_directory: output_dir,
    })
}

#[derive(Serialize)]
pub struct ExportResponse {
    pub success: bool,
    #[serde(rename = "docxFiles")]
    pub docx_files: Vec<String>,
    #[serde(rename = "xlsxFile")]
    pub xlsx_file: String,
    #[serde(rename = "outputDirectory")]
    pub output_directory: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            analyze_docx,
            get_parsed,
            mix_exams,
            export_mixed_exams
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
