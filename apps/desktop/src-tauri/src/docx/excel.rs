// src-tauri/src/docx/excel.rs
//! Excel answer key writer

use rust_xlsxwriter::*;
use std::path::Path;

/// Mixed exam data structure (matches frontend)
#[derive(Debug, serde::Deserialize)]
pub struct MixedExam {
    #[serde(rename = "examCode")]
    pub exam_code: String,
    pub questions: Vec<MixedQuestion>,
}

#[derive(Debug, serde::Deserialize)]
pub struct MixedQuestion {
    #[serde(rename = "originalNumber")]
    pub original_number: usize,
    #[serde(rename = "displayNumber")]
    pub display_number: usize,
    pub stem: Vec<serde_json::Value>,
    pub options: Vec<serde_json::Value>,
    #[serde(rename = "correctAnswer")]
    pub correct_answer: String,
}

/// Write answer key to Excel file
pub fn write_answer_key(
    exams: &[MixedExam],
    original_answers: &[String],
    output_path: &Path,
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    // Create a sheet for each exam variant
    for exam in exams {
        let sheet_name = format!("Đề {}", exam.exam_code);
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(&sheet_name)?;

        // Header row
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4F46E5)); // Violet
        worksheet.write_string_with_format(0, 0, "Câu hỏi", &header_format)?;
        worksheet.write_string_with_format(0, 1, "Đáp án", &header_format)?;
        worksheet.write_string_with_format(0, 2, "Câu gốc", &header_format)?;
        worksheet.write_string_with_format(0, 3, "Đáp án gốc", &header_format)?;

        // Data rows
        for (idx, question) in exam.questions.iter().enumerate() {
            let row = (idx + 1) as u32;

            // Question number (display)
            worksheet.write_number(row, 0, question.display_number as f64)?;

            // Correct answer (after shuffle)
            worksheet.write_string(row, 1, &question.correct_answer)?;

            // Original question number
            worksheet.write_number(row, 2, question.original_number as f64)?;

            // Original answer
            let original_idx = question.original_number - 1;
            if let Some(orig_ans) = original_answers.get(original_idx) {
                worksheet.write_string(row, 3, orig_ans)?;
            }
        }

        // Set column widths
        worksheet.set_column_width(0, 12)?;
        worksheet.set_column_width(1, 12)?;
        worksheet.set_column_width(2, 12)?;
        worksheet.set_column_width(3, 12)?;
    }

    workbook.save(output_path)?;
    Ok(())
}
