// src-tauri/src/docx/header_template.rs
//! Header templates for exam documents
//! Supports different header formats for various exam types

use serde::{Deserialize, Serialize};

/// Standard header template for Vietnamese exam documents
/// Based on common format: School info (left) | Exam info (right)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardHeaderTemplate {
    // Left side
    pub school_name: String,
    pub exam_code: String,
    pub is_official: bool, // "ĐỀ CHÍNH THỨC"
    pub total_pages: u32,  // Calculated during export
    
    // Right side
    pub exam_name: String,      // e.g., "KIỂM TRA GIỮA HKII"
    pub academic_year: String,  // e.g., "2022 - 2023"
    pub subject: String,        // e.g., "LỊCH SỬ- ĐỊA"
    pub grade: String,          // e.g., "LỚP 7"
    pub duration_minutes: u32,
    pub include_distribution_note: bool, // "(Không kể thời gian phát đề)"
}

impl StandardHeaderTemplate {
    /// Create a new header template with default values
    pub fn new(
        school_name: String,
        exam_code: String,
        exam_name: String,
        academic_year: String,
        subject: String,
        grade: String,
        duration_minutes: u32,
    ) -> Self {
        Self {
            school_name,
            exam_code,
            is_official: true,
            total_pages: 1, // Will be calculated
            exam_name,
            academic_year,
            subject,
            grade,
            duration_minutes,
            include_distribution_note: true,
        }
    }
    
    /// Estimate number of pages based on question count
    /// Simple heuristic: ~25-30 questions per page depending on images
    pub fn estimate_pages(question_count: usize) -> u32 {
        let pages = (question_count as f32 / 28.0).ceil() as u32;
        pages.max(1) // At least 1 page
    }
    
    /// Format page count as Vietnamese text (e.g., "02", "10")
    pub fn format_page_count(pages: u32) -> String {
        format!("{:02}", pages)
    }
}
