use regex::Regex;
use std::collections::HashMap;

use crate::docx::model::{OptionItem, ParsedDoc, Question, Segment};
use crate::docx::validator::{LabelRunStyle, LabeledOptionRuns};
use crate::docx::ExtractedAsset;

/// Parse a list of paragraph texts (already extracted from the DOCX)
/// into a `ParsedDoc` following the trắc nghiệm rules.
///
/// - Question start: /^(Câu|Question)\s+(\d+)\./
/// - Options: labels A..F or #A..#F followed by a dot, possibly
///   multiple options in a single paragraph (e.g. "A. ...\tB. ...").
/// - Locked option: label prefixed with '#', e.g. "#A.".
/// - Everything is represented as `Segment::Text` for now.
pub fn parse_paragraphs(paragraphs: &[String]) -> ParsedDoc {
    // Question start: "Câu 1." or "Question 1."
    let question_re = Regex::new(r"^(Câu|Question)\s+(\d+)\.").unwrap();
    let option_re = Regex::new(r"(?P<label>#?[A-F])\.").unwrap();

    let mut questions: Vec<Question> = Vec::new();
    let mut current_question: Option<Question> = None;

    for raw_p in paragraphs {
        let p = raw_p.trim();
        if p.is_empty() {
            continue;
        }

        // 1. Detect question start
        if let Some(caps) = question_re.captures(p) {
            // Close previous question if any
            if let Some(q) = current_question.take() {
                // Only keep questions that have at least one option
                if !q.options.is_empty() {
                    questions.push(q);
                }
            }

            let number: u32 = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);

            // Remove the matched prefix from the text to get the stem text
            let end = caps.get(0).unwrap().end();
            let stem_text = p[end..].trim();

            let mut stem_segments = Vec::new();
            if !stem_text.is_empty() {
                stem_segments.push(Segment::Text {
                    text: stem_text.to_string(),
                });
            }

            current_question = Some(Question {
                number,
                stem: stem_segments,
                options: Vec::new(),
                // Temporary: will be filled from locked or first option label if needed later
                correct_label: String::new(),
            });

            continue;
        }

        // 2. If this paragraph is not a question start, check if it belongs
        //    to the current question as options or stem continuation.
        if let Some(ref mut q) = current_question {
            // Find all option labels in this paragraph
            let matches_vec: Vec<(usize, usize, String)> = option_re
                .captures_iter(p)
                .filter_map(|caps| {
                    let m = caps.get(0)?;
                    let label = caps
                        .name("label")
                        .map(|l| l.as_str().to_string())?;
                    Some((m.start(), m.end(), label))
                })
                .collect();

            if matches_vec.is_empty() {
                // No options found: treat as stem continuation (text only)
                q.stem.push(Segment::Text {
                    text: p.to_string(),
                });
                continue;
            }

            // There are one or more options in this paragraph.
            // Split the paragraph into segments per option.
            // Example: "A. Foo\tB. Bar" -> [A => "Foo", B => "Bar"]
            let mut local_options: Vec<OptionItem> = Vec::new();
            let para_len = p.len();

            let mut idx = 0;
            while idx < matches_vec.len() {
                let (_start, end, raw_label) = &matches_vec[idx];
                let is_locked = raw_label.starts_with('#');
                let label = if is_locked {
                    raw_label[1..].to_string()
                } else {
                    raw_label.clone()
                };

                let content_start = *end;
                let content_end = if idx + 1 < matches_vec.len() {
                    matches_vec[idx + 1].0
                } else {
                    para_len
                };

                let raw_content = p[content_start..content_end].trim();

                let content_segments = if raw_content.is_empty() {
                    Vec::new()
                } else {
                    vec![Segment::Text {
                        text: raw_content.to_string(),
                    }]
                };

                local_options.push(OptionItem {
                    label: label.clone(),
                    locked: is_locked,
                    content: content_segments,
                });

                // If there is a locked option and the question has no
                // correct_label yet, fill it.
                if is_locked && q.correct_label.is_empty() {
                    q.correct_label = label.clone();
                }

                idx += 1;
            }

            q.options.extend(local_options);
        }
    }

    // Push the last question if valid
    if let Some(q) = current_question {
        if !q.options.is_empty() {
            questions.push(q);
        }
    }

    ParsedDoc { questions }
}

/// Very lightweight XML walker that extracts plain paragraph texts from
/// `word/document.xml` by concatenating all `<w:t>` contents inside each
/// `<w:p>`. This is text-only and ignores styling, math and images. It is
/// mainly a bridge to reuse the regex-based `parse_paragraphs` on real
/// DOCX XML.
pub fn parse_document_xml_to_parsed_doc(document_xml: &str) -> ParsedDoc {
    let paragraphs = extract_plain_paragraph_texts(document_xml);
    parse_paragraphs(&paragraphs)
}

fn extract_plain_paragraph_texts(xml: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut cursor = 0;

    loop {
        let start_rel = match xml[cursor..].find("<w:p") {
            Some(idx) => idx,
            None => break,
        };
        let start = cursor + start_rel;

        let end_rel = match xml[start..].find("</w:p>") {
            Some(idx) => idx,
            None => break,
        };
        let end = start + end_rel + "</w:p>".len();

        let block = &xml[start..end];
        let text = extract_text_from_w_p(block);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }

        cursor = end;
    }

    result
}

fn extract_text_from_w_p(block: &str) -> String {
    let mut result = String::new();
    let mut cursor = 0;

    loop {
        let start_rel = match block[cursor..].find("<w:t") {
            Some(idx) => idx,
            None => break,
        };
        let start = cursor + start_rel;

        let gt_rel = match block[start..].find('>') {
            Some(idx) => idx,
            None => break,
        };
        let content_start = start + gt_rel + 1;

        let end_tag_rel = match block[content_start..].find("</w:t>") {
            Some(idx) => idx,
            None => break,
        };
        let content_end = content_start + end_tag_rel;

        let fragment = &block[content_start..content_end];
        let fragment = fragment
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");

        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&fragment);

        cursor = content_end + "</w:t>".len();
    }

    result
}

#[derive(Debug, Clone)]
struct RunInfo {
    text: String,
    underline: bool,
    color: Option<String>,
}

fn extract_runs_from_w_p(block: &str) -> Vec<RunInfo> {
    let mut runs = Vec::new();
    let mut cursor = 0;

    let underline_re = Regex::new(r"<w:u\b[^>]*>").unwrap();
    let color_re = Regex::new(r#"<w:color[^>]*w:val=\"([^\"]+)\""#).unwrap();

    loop {
        let start_rel = match block[cursor..].find("<w:r") {
            Some(idx) => idx,
            None => break,
        };
        let start = cursor + start_rel;

        let end_rel = match block[start..].find("</w:r>") {
            Some(idx) => idx + "</w:r>".len(),
            None => break,
        };
        let end = start + end_rel;

        let r_block = &block[start..end];

        let text = extract_text_from_w_p(r_block).trim().to_string();
        if text.is_empty() {
            cursor = end;
            continue;
        }

        let underline = underline_re
            .find_iter(r_block)
            .any(|m| {
                let tag = &r_block[m.start()..m.end()];
                !(tag.contains("w:val=\"none\"") || tag.contains("w:val='none'"))
            });

        let color = color_re
            .captures(r_block)
            .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()));

        runs.push(RunInfo {
            text,
            underline,
            color,
        });

        cursor = end;
    }

    runs
}

/// Scan `document.xml` and collect styled label runs for each question
/// based on the same text patterns used by `parse_paragraphs`.
///
/// For each question number, returns a vector of `LabeledOptionRuns` whose
/// `runs` contain the underline/color information for the option label
/// (e.g. the run whose text is exactly "A." or "#A.").
pub fn collect_labeled_option_runs(document_xml: &str) -> HashMap<u32, Vec<LabeledOptionRuns>> {
    let question_re = Regex::new(r"^(Câu|Question)\s+(\d+)\.").unwrap();
    let option_label_re = Regex::new(r"^(?P<label>#?[A-F])\.").unwrap();

    let mut result: HashMap<u32, Vec<LabeledOptionRuns>> = HashMap::new();

    let mut cursor = 0;
    let mut current_question: Option<u32> = None;

    loop {
        let start_rel = match document_xml[cursor..].find("<w:p") {
            Some(idx) => idx,
            None => break,
        };
        let start = cursor + start_rel;

        let end_rel = match document_xml[start..].find("</w:p>") {
            Some(idx) => idx + "</w:p>".len(),
            None => break,
        };
        let end = start + end_rel;

        let block = &document_xml[start..end];
        let text = extract_text_from_w_p(block);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            cursor = end;
            continue;
        }

        // Detect question start
        if let Some(caps) = question_re.captures(trimmed) {
            let number: u32 = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            current_question = Some(number);
            cursor = end;
            continue;
        }

        let q_number = match current_question {
            Some(n) => n,
            None => {
                cursor = end;
                continue;
            }
        };

        // For this paragraph, inspect each run and collect those whose
        // text looks like a label (e.g. "A." or "#A.").
        let run_infos = extract_runs_from_w_p(block);
        if run_infos.is_empty() {
            cursor = end;
            continue;
        }

        let entry = result.entry(q_number).or_insert_with(Vec::new);

        for run in run_infos {
            let candidate = run.text.trim();
            if candidate.is_empty() {
                continue;
            }

            if let Some(caps) = option_label_re.captures(candidate) {
                let raw_label = caps
                    .name("label")
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();

                let label = if raw_label.starts_with('#') {
                    raw_label[1..].to_string()
                } else {
                    raw_label.clone()
                };

                if label.is_empty() {
                    continue;
                }

                // Find or create entry for this label
                if let Some(existing) = entry.iter_mut().find(|o| o.label == label) {
                    existing.runs.push(LabelRunStyle {
                        underline: run.underline,
                        color: run.color.clone(),
                    });
                } else {
                    entry.push(LabeledOptionRuns {
                        label: label.clone(),
                        runs: vec![LabelRunStyle {
                            underline: run.underline,
                            color: run.color.clone(),
                        }],
                    });
                }
            }
        }

        cursor = end;
    }

    result
}

/// Inline piece inside a paragraph before being converted to high-level
/// `Segment`s. This is intended to be produced by the lower-level DOCX
/// XML walker:
/// - Text pieces come from normal `w:t` runs.
/// - Math pieces come from `m:oMath` or `m:oMathPara` nodes, with
///   `omml` storing the raw OMML XML.
/// - Image pieces correspond to inline images (`wp:inline` inside
///   `w:drawing`). Asset mapping is resolved by `build_segments_from_pieces`
///   using the global order of appearance.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum InlinePiece {
    Text(String),
    Math { omml: String },
    Image,
}

/// Convert a sequence of inline pieces in document order into
/// high-level `Segment`s, mapping images to the extracted assets
/// using their order of appearance in the whole document.
///
/// - `assets`: list returned from `assets::extract_media`.
/// - `next_asset_index`: mutable cursor shared across the whole
///   document; each time an `InlinePiece::Image` is seen, the
///   corresponding asset is taken from `assets[*next_asset_index]`
///   (if available) and the cursor is incremented.
/// - If there are more images than assets, remaining images will
///   still produce `Segment::Image` with an empty `asset_path`.
#[allow(dead_code)]
pub fn build_segments_from_pieces(
    pieces: &[InlinePiece],
    assets: &[ExtractedAsset],
    next_asset_index: &mut usize,
) -> Vec<Segment> {
    let mut segments = Vec::new();

    for piece in pieces {
        match piece {
            InlinePiece::Text(text) => {
                if !text.is_empty() {
                    segments.push(Segment::Text {
                        text: text.clone(),
                    });
                }
            }
            InlinePiece::Math { omml } => {
                segments.push(Segment::Math {
                    omml: omml.clone(),
                });
            }
            InlinePiece::Image => {
                let asset_path = if *next_asset_index < assets.len() {
                    let p = &assets[*next_asset_index].absolute_path;
                    *next_asset_index += 1;
                    p.to_string_lossy().to_string()
                } else {
                    String::new()
                };

                segments.push(Segment::Image { asset_path });
            }
        }
    }

    segments
}
