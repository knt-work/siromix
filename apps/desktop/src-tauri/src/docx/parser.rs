use regex::Regex;

use crate::docx::model::{OptionItem, ParsedDoc, Question, Segment};

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
