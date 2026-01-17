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
pub fn parse_document_xml_to_parsed_doc(
    document_xml: &str,
    assets: &[ExtractedAsset],
) -> ParsedDoc {
    let question_re = Regex::new(r"^(Câu|Question)\s+(\d+)\.").unwrap();
    let option_re = Regex::new(r"^(?P<label>#?[A-F])\s*\.").unwrap();

    let mut questions: Vec<Question> = Vec::new();
    let mut current_question: Option<Question> = None;
    let mut cursor = 0;
    // Global cursor for mapping images (both <w:drawing> and <w:object>)
    // to extracted media assets by order of appearance.
    let mut next_asset_index: usize = 0;

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
        let segments = extract_segments_from_paragraph(block, assets, &mut next_asset_index);
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
                // Check if there are multiple options in this paragraph (e.g., "C. ... D. ...")
                // by finding all option patterns in the plain text
                let option_positions: Vec<_> = option_re.find_iter(trimmed).collect();
                
                if option_positions.len() > 1 {
                    // Multiple options in same paragraph - need to split
                    for (i, option_match) in option_positions.iter().enumerate() {
                        let option_start = option_match.start();
                        let option_end = if i + 1 < option_positions.len() {
                            option_positions[i + 1].start()
                        } else {
                            trimmed.len()
                        };
                        
                        // Extract label for this option
                        let option_text = &trimmed[option_start..option_end];
                        if let Some(label_caps) = option_re.captures(option_text) {
                            let raw_label = label_caps
                                .name("label")
                                .map(|m| m.as_str().to_string())
                                .unwrap_or_default();

                            let is_locked = raw_label.starts_with('#');
                            let label = if is_locked {
                                raw_label[1..].to_string()
                            } else {
                                raw_label
                            };
                            
                            // For multi-option paragraphs, extract segments for this specific option
                            // This is approximate - we take segments proportionally by character position
                            let prefix_len = label_caps.get(0).unwrap().end();
                            let option_content = &option_text[prefix_len..].trim();
                            
                            // Create a text segment for this option's content
                            let content_segments = vec![Segment::Text {
                                text: option_content.to_string(),
                                raw_xml: String::new(), // TODO: Extract proper raw XML
                            }];

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
                    }
                } else {
                    // Single option - process normally
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
/// - <w:drawing>...</w:drawing> or <w:object>...</w:object> → Segment::Image
///   Images (including OLE Equation objects with VML/v:imagedata) are mapped
///   to extracted media assets purely by global order of appearance.

/// Extract ALL text from ALL <w:t> elements within a single <w:r> run block.
/// This handles cases where Word splits text into multiple <w:t> elements.
/// 
/// Special handling for `<w:t xml:space="preserve">` which indicates that
/// whitespace should be preserved. If the content is empty but has this
/// attribute, it represents a space character.
fn extract_all_text_from_run(run_block: &str) -> String {
    let mut result = String::new();
    let mut cursor = 0;

    loop {
        // Find next <w:t> or <w:t ...>
        let start_rel = match run_block[cursor..].find("<w:t") {
            Some(idx) => idx,
            None => break,
        };
        let start = cursor + start_rel;
        
        // Make sure it's actually <w:t> or <w:t ...>, not <w:tab> etc.
        let after_wt = start + "<w:t".len();
        if after_wt < run_block.len() {
            let next_char = &run_block[after_wt..after_wt + 1];
            if next_char != ">" && next_char != " " {
                // It's <w:tab> or similar, skip
                cursor = after_wt;
                continue;
            }
        }

        // Find end of opening tag
        let gt_rel = match run_block[start..].find('>') {
            Some(idx) => idx,
            None => break,
        };
        let tag_content = &run_block[start..start + gt_rel];
        let content_start = start + gt_rel + 1;

        // Find closing </w:t>
        let end_tag_rel = match run_block[content_start..].find("</w:t>") {
            Some(idx) => idx,
            None => break,
        };
        let content_end = content_start + end_tag_rel;

        // Extract and decode text
        let fragment = &run_block[content_start..content_end];
        
        // Check if xml:space="preserve" is present
        let has_preserve_space = tag_content.contains("xml:space=\"preserve\"") 
            || tag_content.contains("xml:space='preserve'");
        
        if fragment.is_empty() && has_preserve_space {
            // Empty <w:t xml:space="preserve"></w:t> represents a space
            result.push(' ');
        } else {
            let decoded = fragment
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&")
                .replace("&apos;", "'")
                .replace("&quot;", "\"");
            result.push_str(&decoded);
        }
        
        cursor = content_end + "</w:t>".len();
    }

    result
}

/// Helper function to find the start of a <w:r> tag (not <w:rPr>)
fn find_run_start(block: &str, before_pos: usize) -> usize {
    let mut search_end = before_pos;
    loop {
        if let Some(idx) = block[..search_end].rfind("<w:r") {
            // Check the character after "<w:r"
            let check_pos = idx + "<w:r".len();
            if check_pos < block.len() {
                let ch = &block[check_pos..check_pos + 1];
                // Valid <w:r> tag if followed by '>' or ' '
                if ch == ">" || ch == " " {
                    return idx;
                }
                // Otherwise it's <w:rPr> or similar, keep searching backwards
                search_end = idx;
            } else {
                return idx;
            }
        } else {
            return before_pos; // Not found, use fallback
        }
    }
}

fn extract_segments_from_paragraph(
    block: &str,
    assets: &[ExtractedAsset],
    next_asset_index: &mut usize,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut cursor = 0;
    let mut pending_text = String::new();
    let mut pending_raw_xml = String::new();

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
        let next_drawing = block[cursor..].find("<w:drawing");
        let next_object = block[cursor..].find("<w:object");

        // Find which comes first
        let (element_type, offset) = match (next_text, next_math, next_drawing, next_object) {
            (Some(t), None, None, None) => ("text", t),
            (None, Some(m), None, None) => ("math", m),
            (None, None, Some(d), None) => ("drawing", d),
            (None, None, None, Some(o)) => ("object", o),
            (Some(t), Some(m), None, None) => {
                let min_offset = t.min(m);
                if min_offset == t { ("text", t) } else { ("math", m) }
            }
            (Some(t), None, Some(d), None) => {
                let min_offset = t.min(d);
                if min_offset == t { ("text", t) } else { ("drawing", d) }
            }
            (Some(t), None, None, Some(o)) => {
                let min_offset = t.min(o);
                if min_offset == t { ("text", t) } else { ("object", o) }
            }
            (None, Some(m), Some(d), None) => {
                let min_offset = m.min(d);
                if min_offset == m { ("math", m) } else { ("drawing", d) }
            }
            (None, Some(m), None, Some(o)) => {
                let min_offset = m.min(o);
                if min_offset == m { ("math", m) } else { ("object", o) }
            }
            (None, None, Some(d), Some(o)) => {
                let min_offset = d.min(o);
                if min_offset == d { ("drawing", d) } else { ("object", o) }
            }
            (Some(t), Some(m), Some(d), None) => {
                let min_offset = t.min(m).min(d);
                if min_offset == t {
                    ("text", t)
                } else if min_offset == m {
                    ("math", m)
                } else {
                    ("drawing", d)
                }
            }
            (Some(t), Some(m), None, Some(o)) => {
                let min_offset = t.min(m).min(o);
                if min_offset == t {
                    ("text", t)
                } else if min_offset == m {
                    ("math", m)
                } else {
                    ("object", o)
                }
            }
            (Some(t), None, Some(d), Some(o)) => {
                let min_offset = t.min(d).min(o);
                if min_offset == t {
                    ("text", t)
                } else if min_offset == d {
                    ("drawing", d)
                } else {
                    ("object", o)
                }
            }
            (None, Some(m), Some(d), Some(o)) => {
                let min_offset = m.min(d).min(o);
                if min_offset == m {
                    ("math", m)
                } else if min_offset == d {
                    ("drawing", d)
                } else {
                    ("object", o)
                }
            }
            (Some(t), Some(m), Some(d), Some(o)) => {
                let min_offset = t.min(m).min(d).min(o);
                if min_offset == t {
                    ("text", t)
                } else if min_offset == m {
                    ("math", m)
                } else if min_offset == d {
                    ("drawing", d)
                } else {
                    ("object", o)
                }
            }
            (None, None, None, None) => break,
        };

        let start = cursor + offset;

        match element_type {
            "text" => {
                // Find the <w:r> that CONTAINS this <w:t> by searching BACKWARDS
                // But we need to be careful not to find a run that was already processed
                // 
                // Better approach: Find the enclosing </w:r> FORWARD from <w:t>
                // and then find the matching <w:r> that starts AFTER cursor
                
                // First, find where this <w:t> ends
                let gt_rel = match block[start..].find('>') {
                    Some(idx) => idx,
                    None => break,
                };
                let content_start = start + gt_rel + 1;
                
                let end_tag_rel = match block[content_start..].find("</w:t>") {
                    Some(idx) => idx,
                    None => break,
                };
                let wt_end = content_start + end_tag_rel + "</w:t>".len();
                
                // Find the </w:r> that closes this run (forward from <w:t>)
                let run_end = match block[wt_end..].find("</w:r>") {
                    Some(idx) => wt_end + idx + "</w:r>".len(),
                    None => wt_end,
                };
                
                // Find the <w:r> that starts this run - search backwards from <w:t> position
                // but ONLY within the range [cursor..start] to avoid finding runs before cursor
                let search_region = &block[cursor..start];
                let run_start = if let Some(r_idx) = search_region.rfind("<w:r") {
                    // Verify it's <w:r> or <w:r ...>, not <w:rPr>
                    let abs_idx = cursor + r_idx;
                    let check_pos = abs_idx + "<w:r".len();
                    if check_pos < block.len() {
                        let ch = &block[check_pos..check_pos + 1];
                        if ch == ">" || ch == " " {
                            abs_idx
                        } else {
                            start // Fallback to <w:t> position
                        }
                    } else {
                        abs_idx
                    }
                } else {
                    start // No <w:r> found after cursor, use <w:t> position
                };
                
                // Extract ALL text from this run
                let run_block = &block[run_start..run_end];
                let text_fragment = extract_all_text_from_run(run_block);

                // Capture full <w:r>...</w:r> block with formatting
                let raw_xml = run_block.to_string();

                // Accumulate text directly without adding extra spaces
                pending_text.push_str(&text_fragment);
                pending_raw_xml.push_str(&raw_xml);
                
                cursor = run_end;
            }
            "math" => {
                // Flush pending text before adding math
                if !pending_text.is_empty() {
                    let trimmed = pending_text.trim();
                    if !trimmed.is_empty() {
                        segments.push(Segment::Text {
                            text: trimmed.to_string(),
                            raw_xml: pending_raw_xml.clone(),
                        });
                    }
                    pending_text.clear();
                    pending_raw_xml.clear();
                }

                // Extract full <m:oMath>...</m:oMath> block (preserve OMML)
                let end_rel = match block[start..].find("</m:oMath>") {
                    Some(idx) => idx + "</m:oMath>".len(),
                    None => break,
                };
                let end = start + end_rel;
                let omml = block[start..end].to_string();

                // For rawXml, we only include the <m:oMath>...</m:oMath> itself.
                // DO NOT include text runs after math - they should be processed as separate text segments.
                // This prevents "eating" text that belongs to the next segment.
                //
                // For the leading <w:r> (space before math), we can include it if it's within [cursor..start]
                let search_region = &block[cursor..start];
                let run_start = if let Some(r_idx) = search_region.rfind("<w:r") {
                    let abs_idx = cursor + r_idx;
                    let check_pos = abs_idx + "<w:r".len();
                    if check_pos < block.len() {
                        let ch = &block[check_pos..check_pos + 1];
                        if ch == ">" || ch == " " {
                            abs_idx
                        } else {
                            start
                        }
                    } else {
                        abs_idx
                    }
                } else {
                    start
                };

                let raw_xml = block[run_start..end].to_string();

                segments.push(Segment::Math { 
                    omml,
                    raw_xml,
                });
                
                // Move cursor to just after </m:oMath> - let text runs after be processed normally
                cursor = end;
            }
            "drawing" => {
                // Flush pending text before adding image
                if !pending_text.is_empty() {
                    let trimmed = pending_text.trim();
                    if !trimmed.is_empty() {
                        segments.push(Segment::Text {
                            text: trimmed.to_string(),
                            raw_xml: pending_raw_xml.clone(),
                        });
                    }
                    pending_text.clear();
                    pending_raw_xml.clear();
                }

                // Find the containing <w:r> block
                let run_start = find_run_start(block, start);

                // Find end of <w:drawing>
                let end_rel = match block[start..].find("</w:drawing>") {
                    Some(idx) => idx + "</w:drawing>".len(),
                    None => break,
                };
                let end = start + end_rel;

                // Find end of </w:r>
                let run_end = match block[end..].find("</w:r>") {
                    Some(idx) => end + idx + "</w:r>".len(),
                    None => end,
                };

                let raw_xml = block[run_start..run_end].to_string();

                // Map this drawing to the next extracted media asset (if any)
                let asset_path = if *next_asset_index < assets.len() {
                    let asset = &assets[*next_asset_index];
                    *next_asset_index += 1;
                    // Prefer converted PNG if available (for WMF/EMF files)
                    let path = asset.converted_path
                        .as_ref()
                        .unwrap_or(&asset.absolute_path)
                        .to_string_lossy()
                        .to_string();
                    
                    // Debug log
                    if asset.converted_path.is_some() {
                        println!("[Parser] Using converted PNG: {} (was: {})", 
                            path, asset.file_name);
                    }
                    
                    path
                } else {
                    String::new()
                };

                // Parse dimensions from <wp:extent cx="..." cy="..."/>
                let (width_emu, height_emu) = parse_image_dimensions(&raw_xml);

                if !asset_path.is_empty() {
                    segments.push(Segment::Image { 
                        asset_path,
                        raw_xml,
                        width_emu,
                        height_emu,
                    });
                }

                cursor = run_end;
            }
            "object" => {
                // Flush pending text before adding image (OLE Equation object)
                if !pending_text.is_empty() {
                    let trimmed = pending_text.trim();
                    if !trimmed.is_empty() {
                        segments.push(Segment::Text {
                            text: trimmed.to_string(),
                            raw_xml: pending_raw_xml.clone(),
                        });
                    }
                    pending_text.clear();
                    pending_raw_xml.clear();
                }

                // Find the containing <w:r> block
                let run_start = find_run_start(block, start);

                // Find end of <w:object>
                let end_rel = match block[start..].find("</w:object>") {
                    Some(idx) => idx + "</w:object>".len(),
                    None => break,
                };
                let end = start + end_rel;

                // Find end of </w:r>
                let run_end = match block[end..].find("</w:r>") {
                    Some(idx) => end + idx + "</w:r>".len(),
                    None => end,
                };

                let raw_xml = block[run_start..run_end].to_string();

                // Map this OLE object (which contains <v:imagedata>)
                // to the next extracted media asset (Equation preview image).
                let asset_path = if *next_asset_index < assets.len() {
                    let asset = &assets[*next_asset_index];
                    *next_asset_index += 1;
                    // Prefer converted PNG if available (for WMF/EMF files)
                    let path = asset.converted_path
                        .as_ref()
                        .unwrap_or(&asset.absolute_path)
                        .to_string_lossy()
                        .to_string();
                    
                    // Debug log
                    if asset.converted_path.is_some() {
                        println!("[Parser] Using converted PNG for OLE: {} (was: {})", 
                            path, asset.file_name);
                    }
                    
                    path
                } else {
                    String::new()
                };

                // Parse dimensions from <v:shape style="width:...;height:..."/>
                let (width_emu, height_emu) = parse_image_dimensions(&raw_xml);

                if !asset_path.is_empty() {
                    segments.push(Segment::Image { 
                        asset_path,
                        raw_xml,
                        width_emu,
                        height_emu,
                    });
                }

                cursor = run_end;
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
                raw_xml: pending_raw_xml.clone(),
            });
        }
    }

    segments
}

/// Parse image dimensions from XML (either <wp:extent> for drawings or <v:shape> for objects)
/// 
/// This function tries multiple sources for image dimensions in order of preference:
/// 1. <wp:extent cx="..." cy="..."/> - Main extent in WordprocessingDrawing
/// 2. <a:ext cx="..." cy="..."/> inside <a:xfrm> - Transform extent in DrawingML
/// 3. <v:shape style="width:...;height:..."/> - VML shape for OLE objects
/// 
/// The dimensions are returned in EMUs (English Metric Units).
/// 1 inch = 914400 EMUs, 1 pt = 12700 EMUs, 1 cm = 360000 EMUs
fn parse_image_dimensions(xml: &str) -> (i64, i64) {
    // Priority 1: Try to find <wp:extent cx="..." cy="..."/> for modern drawings
    // This is the "desired" display size set by the user
    if let Some(extent_start) = xml.find("<wp:extent") {
        if let Some(extent_end) = xml[extent_start..].find("/>") {
            let extent_block = &xml[extent_start..extent_start + extent_end];
            let dims = parse_cx_cy_attributes(extent_block);
            if dims.0 > 0 && dims.1 > 0 {
                return dims;
            }
        }
    }
    
    // Priority 2: Try <a:ext> inside <a:xfrm> (DrawingML transform)
    // This is often the actual rendered size
    if let Some(xfrm_start) = xml.find("<a:xfrm") {
        if let Some(xfrm_end_rel) = xml[xfrm_start..].find("</a:xfrm>") {
            let xfrm_block = &xml[xfrm_start..xfrm_start + xfrm_end_rel];
            if let Some(ext_start) = xfrm_block.find("<a:ext") {
                if let Some(ext_end) = xfrm_block[ext_start..].find("/>") {
                    let ext_block = &xfrm_block[ext_start..ext_start + ext_end];
                    let dims = parse_cx_cy_attributes(ext_block);
                    if dims.0 > 0 && dims.1 > 0 {
                        return dims;
                    }
                }
            }
        }
    }
    
    // Priority 3: Try <pic:spPr> -> <a:xfrm> -> <a:ext> (Picture shape properties)
    if let Some(sppr_start) = xml.find("<pic:spPr") {
        if let Some(sppr_end_rel) = xml[sppr_start..].find("</pic:spPr>") {
            let sppr_block = &xml[sppr_start..sppr_start + sppr_end_rel];
            if let Some(ext_start) = sppr_block.find("<a:ext") {
                if let Some(ext_end) = sppr_block[ext_start..].find("/>") {
                    let ext_block = &sppr_block[ext_start..ext_start + ext_end];
                    let dims = parse_cx_cy_attributes(ext_block);
                    if dims.0 > 0 && dims.1 > 0 {
                        return dims;
                    }
                }
            }
        }
    }
    
    // Priority 4: Try to parse from <v:shape style="...width:...;height:..."/> for OLE objects
    if let Some(shape_start) = xml.find("<v:shape") {
        if let Some(style_start) = xml[shape_start..].find("style=\"") {
            let style_val_start = shape_start + style_start + 7;
            if let Some(style_end) = xml[style_val_start..].find('"') {
                let style_block = &xml[style_val_start..style_val_start + style_end];
                let dims = parse_vml_style_dimensions(style_block);
                if dims.0 > 0 && dims.1 > 0 {
                    return dims;
                }
            }
        }
    }
    
    // Default fallback size (1 inch x 1 inch)
    (914400, 914400)
}

/// Parse cx and cy attributes from an XML element string like `<wp:extent cx="123" cy="456"`
fn parse_cx_cy_attributes(element: &str) -> (i64, i64) {
    let width = if let Some(cx_start) = element.find("cx=\"") {
        let cx_val_start = cx_start + 4;
        if let Some(cx_end) = element[cx_val_start..].find('"') {
            element[cx_val_start..cx_val_start + cx_end]
                .parse::<i64>()
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    
    let height = if let Some(cy_start) = element.find("cy=\"") {
        let cy_val_start = cy_start + 4;
        if let Some(cy_end) = element[cy_val_start..].find('"') {
            element[cy_val_start..cy_val_start + cy_end]
                .parse::<i64>()
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    
    (width, height)
}

/// Parse width and height from VML style string like "width:72pt;height:48pt"
/// Supports units: pt (points), in (inches), cm (centimeters), mm (millimeters)
fn parse_vml_style_dimensions(style: &str) -> (i64, i64) {
    let width = parse_vml_dimension(style, "width:");
    let height = parse_vml_dimension(style, "height:");
    (width, height)
}

/// Parse a single dimension value from VML style string
fn parse_vml_dimension(style: &str, prefix: &str) -> i64 {
    if let Some(start) = style.find(prefix) {
        let val_start = start + prefix.len();
        let remaining = &style[val_start..];
        
        // Find the end of the value (next ; or end of string)
        let val_end = remaining.find(';').unwrap_or(remaining.len());
        let value_str = remaining[..val_end].trim();
        
        // Try different units
        if let Some(num_end) = value_str.find("pt") {
            if let Ok(val) = value_str[..num_end].trim().parse::<f64>() {
                return (val * 12700.0) as i64; // 1 pt = 12700 EMUs
            }
        } else if let Some(num_end) = value_str.find("in") {
            if let Ok(val) = value_str[..num_end].trim().parse::<f64>() {
                return (val * 914400.0) as i64; // 1 inch = 914400 EMUs
            }
        } else if let Some(num_end) = value_str.find("cm") {
            if let Ok(val) = value_str[..num_end].trim().parse::<f64>() {
                return (val * 360000.0) as i64; // 1 cm = 360000 EMUs
            }
        } else if let Some(num_end) = value_str.find("mm") {
            if let Ok(val) = value_str[..num_end].trim().parse::<f64>() {
                return (val * 36000.0) as i64; // 1 mm = 36000 EMUs
            }
        } else if let Some(num_end) = value_str.find("px") {
            if let Ok(val) = value_str[..num_end].trim().parse::<f64>() {
                return (val * 9525.0) as i64; // 1 px = 9525 EMUs (at 96 DPI)
            }
        }
    }
    0
}

/// Convert segments to plain text for regex pattern matching.
///
/// Used to detect question/option prefixes while preserving segment structure.
/// Text segments are concatenated directly (no extra spaces added between them).
/// Math segments are represented as single space (so they don't interfere with text matching).
fn segments_to_plain_text(segments: &[Segment]) -> String {
    let mut result = String::new();
    for seg in segments {
        match seg {
            Segment::Text { text, .. } => {
                // Concatenate text directly without adding extra spaces
                // The text already contains proper spacing from the DOCX
                result.push_str(text);
            }
            Segment::Math { .. } => {
                // Represent math as a placeholder space for regex purposes
                // Add space only if needed to separate from previous content
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push(' '); // Placeholder for math
            }
            Segment::Image { .. } => {
                // Images don't contribute to text matching
                // But add space if needed to avoid words sticking together
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
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
///
/// Note: The prefix_len corresponds to the character count from segments_to_plain_text(),
/// where text segments are concatenated directly without extra spaces.
fn trim_prefix_from_segments(segments: &[Segment], prefix_len: usize) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut chars_skipped = 0;

    for seg in segments {
        match seg {
            Segment::Text { text, raw_xml } => {
                if chars_skipped >= prefix_len {
                    // Already skipped enough, keep this segment
                    result.push(seg.clone());
                } else if chars_skipped + text.chars().count() > prefix_len {
                    // Prefix ends in the middle of this text segment
                    // Use char indices for proper Unicode handling
                    let skip_in_this = prefix_len - chars_skipped;
                    let char_boundary: usize = text.char_indices()
                        .nth(skip_in_this)
                        .map(|(idx, _)| idx)
                        .unwrap_or(text.len());
                    let remaining = text[char_boundary..].trim_start().to_string();
                    if !remaining.is_empty() {
                        result.push(Segment::Text { 
                            text: remaining,
                            raw_xml: raw_xml.clone(),
                        });
                    }
                    chars_skipped = prefix_len;
                } else {
                    // This entire text segment is part of the prefix, skip it
                    // Count actual characters (not bytes) for proper Unicode handling
                    chars_skipped += text.chars().count();
                }
            }
            Segment::Math { .. } => {
                // Math occupies 2 spaces in plain text (one before, one placeholder)
                // as per segments_to_plain_text logic
                if chars_skipped >= prefix_len {
                    result.push(seg.clone());
                } else {
                    chars_skipped += 2;
                }
            }
            Segment::Image { .. } => {
                // Images occupy 1 space in plain text (separator space)
                if chars_skipped >= prefix_len {
                    result.push(seg.clone());
                } else {
                    chars_skipped += 1;
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
/// 
/// Handles `xml:space="preserve"` attribute - empty elements with this
/// attribute represent a space character.
fn extract_text_from_w_p(block: &str) -> String {
    let mut result = String::new();
    let mut cursor = 0;

    loop {
        let start_rel = match block[cursor..].find("<w:t") {
            Some(idx) => idx,
            None => break,
        };
        let start = cursor + start_rel;
        
        // Skip <w:tab> and similar
        let after_wt = start + "<w:t".len();
        if after_wt < block.len() {
            let check_end = (after_wt + 1).min(block.len());
            let next_char = &block[after_wt..check_end];
            if !next_char.is_empty() && next_char != ">" && next_char != " " {
                cursor = after_wt;
                continue;
            }
        }

        let gt_rel = match block[start..].find('>') {
            Some(idx) => idx,
            None => break,
        };
        let tag_content = &block[start..start + gt_rel];
        let content_start = start + gt_rel + 1;

        let end_tag_rel = match block[content_start..].find("</w:t>") {
            Some(idx) => idx,
            None => break,
        };
        let content_end = content_start + end_tag_rel;

        let fragment = &block[content_start..content_end];
        
        // Check if xml:space="preserve" is present
        let has_preserve_space = tag_content.contains("xml:space=\"preserve\"") 
            || tag_content.contains("xml:space='preserve'");
        
        if fragment.is_empty() && has_preserve_space {
            // Empty <w:t xml:space="preserve"></w:t> represents a space
            result.push(' ');
        } else {
            let decoded = fragment
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&")
                .replace("&apos;", "'")
                .replace("&quot;", "\"");
            result.push_str(&decoded);
        }

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
    let option_label_re = Regex::new(r"^(?P<label>#?[A-F])\s*(\.|$)").unwrap();

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
                        raw_xml: String::new(), // Legacy function - no raw XML available
                    });
                }
            }
            InlinePiece::Math { omml } => {
                segments.push(Segment::Math {
                    omml: omml.clone(),
                    raw_xml: String::new(), // Legacy function - no raw XML available
                });
            }
            InlinePiece::Image => {
                let asset_path = if *next_asset_index < assets.len() {
                    let asset = &assets[*next_asset_index];
                    *next_asset_index += 1;
                    // Prefer converted PNG if available (for WMF/EMF files)
                    asset.converted_path
                        .as_ref()
                        .unwrap_or(&asset.absolute_path)
                        .to_string_lossy()
                        .to_string()
                } else {
                    String::new()
                };

                segments.push(Segment::Image { 
                    asset_path,
                    raw_xml: String::new(), // Legacy function - no raw XML available
                    width_emu: 0,
                    height_emu: 0,
                });
            }
        }
    }

    segments
}
