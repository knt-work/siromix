/// Utilities for validating questions and detecting the correct answer
/// based on DOCX run styling.
///
/// Rules:
/// - The correct answer is marked on the option LABEL (e.g. "A." or "#A.")
///   using either underline or red color.
/// - We only inspect runs that belong to the label (from the start of the
///   label up to the trailing dot). Higher-level parsing code is responsible
///   for slicing the DOCX runs so that only the label runs are passed here.
/// - Underline: `<w:rPr><w:u w:val != "none" />`
/// - Red color: `<w:rPr><w:color w:val="FF0000" />` (case‑insensitive).

/// Minimal styling info for a single DOCX run within an option label.
#[derive(Debug, Clone)]
pub struct LabelRunStyle {
    pub underline: bool,
    /// Raw color value from `w:color/@w:val`, e.g. "FF0000".
    pub color: Option<String>,
}

impl LabelRunStyle {
    /// Returns true if this run contributes to marking the label as correct
    /// (underline or red color).
    pub fn is_marked(&self) -> bool {
        if self.underline {
            return true;
        }

        if let Some(ref c) = self.color {
            if c.eq_ignore_ascii_case("FF0000") {
                return true;
            }
        }

        false
    }
}

/// A helper structure representing one option's label and the runs that
/// make up that label.
#[derive(Debug, Clone)]
pub struct LabeledOptionRuns {
    /// Logical label for the option, e.g. "A", "B", ...
    /// Callers may strip any leading '#' before populating this field.
    pub label: String,
    /// Runs that belong only to the label text (up to the trailing dot).
    pub runs: Vec<LabelRunStyle>,
}

/// Error codes for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorCode {
    /// No option in the question has a correct-marked label.
    E020CorrectMarkMissing,
    /// More than one option in the question has a correct-marked label.
    E021CorrectMarkMultiple,
}

impl ValidationErrorCode {
    /// Stable string representation for frontend / logging.
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationErrorCode::E020CorrectMarkMissing => "E020_CORRECT_MARK_MISSING",
            ValidationErrorCode::E021CorrectMarkMultiple => "E021_CORRECT_MARK_MULTIPLE",
        }
    }
}

/// A validation error associated with a particular question.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub code: ValidationErrorCode,
    /// Question number in the parsed document.
    pub question_number: u32,
}

/// Determine whether a label (described by its runs) is marked as the
/// correct answer.
pub fn is_label_marked_correct(runs: &[LabelRunStyle]) -> bool {
    runs.iter().any(|r| r.is_marked())
}

/// Given all options for a question (with their label runs), detect which
/// label is marked as correct and enforce that exactly one such label exists.
///
/// Returns `Ok(label)` with the single correct label (e.g. "A") or
/// `Err(ValidationError)` with one of the error codes:
/// - `E020_CORRECT_MARK_MISSING` if no option is marked correct.
/// - `E021_CORRECT_MARK_MULTIPLE` if more than one option is marked.
pub fn detect_correct_label_for_question(
    question_number: u32,
    options: &[LabeledOptionRuns],
) -> Result<String, ValidationError> {
    let mut marked_labels: Vec<String> = Vec::new();

    for opt in options {
        if is_label_marked_correct(&opt.runs) {
            marked_labels.push(opt.label.clone());
        }
    }

    match marked_labels.len() {
        0 => Err(ValidationError {
            code: ValidationErrorCode::E020CorrectMarkMissing,
            question_number,
        }),
        1 => Ok(marked_labels.remove(0)),
        _ => Err(ValidationError {
            code: ValidationErrorCode::E021CorrectMarkMultiple,
            question_number,
        }),
    }
}
