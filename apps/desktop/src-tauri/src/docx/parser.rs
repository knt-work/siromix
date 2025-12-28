use regex::Regex;
use std::collections::HashMap;

use crate::docx::model::{OptionItem, ParsedDoc, Question, Segment};
use crate::docx::validator::{LabelRunStyle, LabeledOptionRuns};
use crate::docx::ExtractedAsset;

/// Parse document.xml into ParsedDoc by extracting segments (text, math, images)
/// from each paragraph while preserving question/option structure.
///
/// Rules:
/// - Each paragraph has ONE role: new question, new option, or continuation
/// - Question starts with "Câu X." or "Question X."
/// - Option starts with "A." / "B." / "C." / "D." / "E." / "F." (or "#A." for locked)
/// - Continuation paragraphs are added to current question stem or option content
pub fn parse_document_xml_to_parsed_doc(document_xml: &str) -> ParsedDoc {
    let question_re = Regex::new(r"^(Câu|Question)\s+(\d+)\.").unwrap();
    let option_re = Regex::new(r"^(?P<label>#?[A-F])\.").unwrap();

    let mut questions: Vec<Question> = Vec::new();
    let mut current_question: Option<Question> = None;
    let mut cursor = 0;

    // Walk through all <w:p> blocks
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
        
        // Extract segments (text, math, images) from this paragraph
        let segments = extract_segments_from_paragraph(block);
        if segments.is_empty() {
            cursor = end;
            continue;
        }

        // Get plain text for pattern matching
        let plain_text = segments_to_plain_text(&segments);
        let trimmed = plain_text.trim();

        // Case 1: New question paragraph (starts with "Câu X." or "Question X.")
        if let Some(caps) = question_re.captures(trimmed) {
            // Save previous question if any
            if let Some(q) = current_question.take() {
                if !q.options.is_empty() {
                    questions.push(q);
                }
            }

            let number: u32 = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);

            // Remove question prefix ("Câu 1. ") from segments to get stem content
            let prefix_end = caps.get(0).unwrap().end();
            let stem_segments = trim_prefix_from_segments(&segments, prefix_end);

            current_question = Some(Question {
                number,
                stem: stem_segments,
                options: Vec::new(),
                correct_label: String::new(),
            });

            cursor = end;
            continue;
        }

        // Case 2: New option paragraph (starts with "A." / "B." / etc.)
        if let Some(caps) = option_re.captures(trimmed) {
            if let Some(ref mut q) = current_question {
                let raw_label = caps
                    .name("label")
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();

                let is_locked = raw_label.starts_with('#');
                let label = if is_locked {
                    raw_label[1..].to_string()
                } else {
                    raw_label
                };

                // Remove option prefix ("A. ") from segments to get content
                let prefix_end = caps.get(0).unwrap().end();
                let content_segments = trim_prefix_from_segments(&segments, prefix_end);

                q.options.push(OptionItem {
                    label: label.clone(),
                    locked: is_locked,
                    content: content_segments,
                });

                // If this is a locked option (e.g., "#A."), set as correct answer
                if is_locked && q.correct_label.is_empty() {
                    q.correct_label = label;
                }
            }

            cursor = end;
            continue;
        }

        // Case 3: Continuation paragraph (no question/option prefix)
        // Add to current question stem or current option content
        if let Some(ref mut q) = current_question {
            if q.options.is_empty() {
                // No options yet: add to stem
                q.stem.extend(segments);
            } else {
                // Has options: add to last option's content
                if let Some(last_option) = q.options.last_mut() {
                    last_option.content.extend(segments);
                }
            }
        }

        cursor = end;
    }

    // Push last question if valid
    if let Some(q) = current_question {
        if !q.options.is_empty() {
            questions.push(q);
        }
    }

    ParsedDoc { questions }
}

/// Extract segments (Text, Math, Image) from a single <w:p> block preserving order.
///
/// Walks through the paragraph XML and creates appropriate Segment variants:
/// - <w:t>text</w:t> → Segment::Text
/// - <m:oMath>...</m:oMath> → Segment::Math (preserves full OMML for frontend)
/// - <w:drawing>...</w:drawing> → Segment::Image (extracts rId, needs .rels mapping)
fn extract_segments_from_paragraph(block: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut cursor = 0;
    let mut pending_text = String::new();

    loop {
        // Look for next interesting element: <w:t>, <m:oMath>, or <w:drawing>
        // Note: Must search for "<w:t>" or "<w:t " to avoid matching "<w:tab"
        let next_text_space = block[cursor..].find("<w:t ");
        let next_text_gt = block[cursor..].find("<w:t>");
        let next_text = match (next_text_space, next_text_gt) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let next_math = block[cursor..].find("<m:oMath");
        let next_image = block[cursor..].find("<w:drawing");

        // Find which comes first
        let (element_type, offset) = match (next_text, next_math, next_image) {
            (Some(t), None, None) => ("text", t),
            (None, Some(m), None) => ("math", m),
            (None, None, Some(i)) => ("image", i),
            (Some(t), Some(m), None) => {
                if t < m {
                    ("text", t)
                } else {
                    ("math", m)
                }
            }
            (Some(t), None, Some(i)) => {
                if t < i {
                    ("text", t)
                } else {
                    ("image", i)
                }
            }
            (None, Some(m), Some(i)) => {
                if m < i {
                    ("math", m)
                } else {
                    ("image", i)
                }
            }
            (Some(t), Some(m), Some(i)) => {
                let min_offset = t.min(m).min(i);
                if min_offset == t {
                    ("text", t)
                } else if min_offset == m {
                    ("math", m)
                } else {
                    ("image", i)
                }
            }
            (None, None, None) => break,
        };

        let start = cursor + offset;

        match element_type {
            "text" => {
                // Extract text content from <w:t>text</w:t>
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
                
                // Decode XML entities
                let fragment = fragment
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&");

                pending_text.push_str(&fragment);
                cursor = content_end + "</w:t>".len();
            }
            "math" => {
                // Flush pending text before adding math
                if !pending_text.is_empty() {
                    let trimmed = pending_text.trim();
                    if !trimmed.is_empty() {
                        segments.push(Segment::Text {
                            text: trimmed.to_string(),
                        });
                    }
                    pending_text.clear();
                }

                // Extract full <m:oMath>...</m:oMath> block (preserve OMML)
                let end_rel = match block[start..].find("</m:oMath>") {
                    Some(idx) => idx + "</m:oMath>".len(),
                    None => break,
                };
                let end = start + end_rel;
                let omml = block[start..end].to_string();

                segments.push(Segment::Math { omml });
                cursor = end;
            }
            "image" => {
                // Flush pending text before adding image
                if !pending_text.is_empty() {
                    let trimmed = pending_text.trim();
                    if !trimmed.is_empty() {
                        segments.push(Segment::Text {
                            text: trimmed.to_string(),
                        });
                    }
                    pending_text.clear();
                }

                // Extract image reference from <w:drawing>...</w:drawing>
                let end_rel = match block[start..].find("</w:drawing>") {
                    Some(idx) => idx + "</w:drawing>".len(),
                    None => break,
                };
                let end = start + end_rel;
                
                // Extract rId from the drawing block (needs .rels mapping later)
                if let Some(asset_path) = extract_image_path_from_drawing(&block[start..end]) {
                    segments.push(Segment::Image { asset_path });
                }
                
                cursor = end;
            }
            _ => break,
        }
    }

    // Flush remaining text
    if !pending_text.is_empty() {
        let trimmed = pending_text.trim();
        if !trimmed.is_empty() {
            segments.push(Segment::Text {
                text: trimmed.to_string(),
            });
        }
    }

    segments
}

/// Extract image asset path from a <w:drawing> block.
/// 
/// Looks for r:embed="rIdX" in <a:blip> element.
/// TODO: Parse document.xml.rels to map rId → actual media file path.
/// For now returns None (placeholder implementation).
fn extract_image_path_from_drawing(_drawing_block: &str) -> Option<String> {
    // TODO: Parse blip:embed rId, look up in document.xml.rels, map to media/imageN.ext
    // For now, return None since we need the full extraction logic with rels parsing
    None
}

/// Convert segments to plain text for regex pattern matching.
///
/// Used to detect question/option prefixes while preserving segment structure.
/// Math segments are represented as single space (so they don't interfere with text matching).
fn segments_to_plain_text(segments: &[Segment]) -> String {
    let mut result = String::new();
    for seg in segments {
        match seg {
            Segment::Text { text } => {
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push_str(text);
            }
            Segment::Math { .. } => {
                // Represent math as a placeholder space for regex purposes
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
            }
            Segment::Image { .. } => {
                // Images don't contribute to text matching
            }
        }
    }
    result
}

/// Remove prefix characters from segments.
///
/// Used after detecting question/option prefix (e.g., "Câu 1. " or "A. ")
/// to get the actual content segments without the prefix.
///
/// # Arguments
/// * `segments` - Original segments from paragraph
/// * `prefix_len` - Number of characters to skip (from plain text representation)
fn trim_prefix_from_segments(segments: &[Segment], prefix_len: usize) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut chars_skipped = 0;

    for seg in segments {
        match seg {
            Segment::Text { text } => {
                if chars_skipped >= prefix_len {
                    // Already skipped enough, keep this segment
                    result.push(seg.clone());
                } else if chars_skipped + text.len() > prefix_len {
                    // Prefix ends in the middle of this text segment
                    let skip_in_this = prefix_len - chars_skipped;
                    let remaining = text[skip_in_this..].trim_start().to_string();
                    if !remaining.is_empty() {
                        result.push(Segment::Text { text: remaining });
                    }
                    chars_skipped = prefix_len;
                } else {
                    // This entire text segment is part of the prefix, skip it
                    chars_skipped += text.len() + 1; // +1 for space added in plain text
                }
            }
            Segment::Math { .. } => {
                // Math occupies 1 space in plain text
                if chars_skipped >= prefix_len {
                    result.push(seg.clone());
                } else {
                    chars_skipped += 1;
                }
            }
            Segment::Image { .. } => {
                // Images don't occupy space in plain text
                if chars_skipped >= prefix_len {
                    result.push(seg.clone());
                }
            }
        }
    }

    result
}

/// Extract plain text from a block by concatenating all <w:t> elements.
///
/// Used by styling-aware functions (like collect_labeled_option_runs)
/// that need plain text for pattern matching while preserving run boundaries.
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
    // Chấp nhận cả trường hợp nhãn chỉ là chữ cái ("D") lẫn "D." trong cùng một run.
    // Điều này xử lý các tình huống DOCX tách "D" và "." thành hai run khác nhau.
    let option_label_re = Regex::new(r"^(?P<label>#?[A-F])(\.|$)").unwrap();

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
