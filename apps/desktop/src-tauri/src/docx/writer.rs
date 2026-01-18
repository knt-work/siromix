// src-tauri/src/docx/writer.rs
//! DOCX writer with manual OMML injection
//! Generates valid OpenXML .docx files with proper formatting

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

use super::model::{Question, Segment};
use super::config::NghiDinh30;

/// Exam writer that generates a complete DOCX file
pub struct ExamWriter {
    pub exam_code: String,
    pub questions: Vec<Question>,
    pub exam_title: String,
    pub subject: String,
    pub duration_minutes: u32,
    pub assets_dir: PathBuf,
    // Header metadata
    pub school_name: String,
    pub exam_name: String,
    pub academic_year: String,
    pub grade: String,
}

/// Image information for embedding
#[derive(Debug, Clone)]
struct ImageInfo {
    rel_id: String,
    path: PathBuf,
    extension: String,
    width_emu: i64,  // Width in EMUs (English Metric Units)
    height_emu: i64, // Height in EMUs
}

impl ExamWriter {
    /// Write DOCX file to disk
    pub fn write_to_file(&self, output_path: &Path) -> Result<(), std::io::Error> {
        let file = File::create(output_path)?;
        let mut zip = ZipWriter::new(BufWriter::new(file));
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // Collect all images from questions
        let image_map = self.collect_images();

        // 1. [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(self.generate_content_types().as_bytes())?;

        // 2. _rels/.rels
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(self.generate_root_rels().as_bytes())?;

        // 3. word/document.xml (main content)
        zip.start_file("word/document.xml", options)?;
        zip.write_all(self.generate_document_xml(&image_map).as_bytes())?;

        // 4. word/_rels/document.xml.rels
        zip.start_file("word/_rels/document.xml.rels", options)?;
        zip.write_all(self.generate_document_rels(&image_map).as_bytes())?;

        // 5. word/styles.xml
        zip.start_file("word/styles.xml", options)?;
        zip.write_all(self.generate_styles_xml().as_bytes())?;

        // 6. word/footer1.xml (page numbers)
        zip.start_file("word/footer1.xml", options)?;
        zip.write_all(self.generate_footer_xml().as_bytes())?;

        // 7. Embed images
        self.embed_images(&mut zip, options, &image_map)?;

        zip.finish()?;
        Ok(())
    }

    /// Collect all unique images from questions and assign relationship IDs
    fn collect_images(&self) -> HashMap<String, ImageInfo> {
        let mut image_map = HashMap::new();
        let mut rel_counter = 1;

        for question in &self.questions {
            // Check stem segments
            for segment in &question.stem {
                if let Segment::Image { asset_path, .. } = segment {
                    if !image_map.contains_key(asset_path) {
                        if let Some(info) = self.create_image_info(asset_path, rel_counter) {
                            image_map.insert(asset_path.clone(), info);
                            rel_counter += 1;
                        }
                    }
                }
            }

            // Check option segments
            for option in &question.options {
                for segment in &option.content {
                    if let Segment::Image { asset_path, .. } = segment {
                        if !image_map.contains_key(asset_path) {
                            if let Some(info) = self.create_image_info(asset_path, rel_counter) {
                                image_map.insert(asset_path.clone(), info);
                                rel_counter += 1;
                            }
                        }
                    }
                }
            }
        }

        image_map
    }

    /// Create ImageInfo from asset path
    fn create_image_info(&self, asset_path: &str, rel_id: usize) -> Option<ImageInfo> {
        let path = PathBuf::from(asset_path);
        if !path.exists() {
            return None;
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();

        // Read image dimensions
        let (width_emu, height_emu) = match image::image_dimensions(&path) {
            Ok((width, height)) => {
                // Convert pixels to EMUs (1 pixel = 9525 EMUs at 96 DPI)
                let width_emu = (width as i64) * 9525;
                let height_emu = (height as i64) * 9525;
                (width_emu, height_emu)
            }
            Err(_) => {
                // Fallback to default size if can't read dimensions
                (914400, 914400) // 1 inch x 1 inch
            }
        };

        Some(ImageInfo {
            rel_id: format!("rId{}", rel_id),
            path,
            extension,
            width_emu,
            height_emu,
        })
    }

    /// Generate [Content_Types].xml
    fn generate_content_types(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Default Extension="png" ContentType="image/png"/>
    <Default Extension="jpeg" ContentType="image/jpeg"/>
    <Default Extension="jpg" ContentType="image/jpeg"/>
    <Default Extension="wmf" ContentType="image/x-wmf"/>
    <Default Extension="emf" ContentType="image/x-emf"/>
    <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
    <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
    <Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>
</Types>"#
        )
    }

    /// Generate _rels/.rels
    fn generate_root_rels(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#
            .to_string()
    }

    /// Generate word/document.xml with questions and OMML
    fn generate_document_xml(&self, image_map: &HashMap<String, ImageInfo>) -> String {
        let mut doc = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
            xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
            xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
            xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
    <w:body>"#,
        );

        // Header
        doc.push_str(&self.generate_header());

        // Questions
        for (idx, question) in self.questions.iter().enumerate() {
            doc.push_str(&self.generate_question_xml(idx + 1, question, image_map));
        }

        // Add section properties with page setup (A4) and footer reference
        doc.push_str(&self.generate_section_properties());

        doc.push_str(
            r#"
    </w:body>
</w:document>"#,
        );
        doc
    }

    /// Generate header section as a table with left and right columns
    fn generate_header(&self) -> String {
        use super::header_template::StandardHeaderTemplate;
        
        let total_pages = StandardHeaderTemplate::estimate_pages(self.questions.len());
        let page_text = StandardHeaderTemplate::format_page_count(total_pages);
        
        let font = NghiDinh30::FONT_NAME;
        let size = NghiDinh30::FONT_SIZE_HEADER;
        let line_spacing = NghiDinh30::HEADER_LINE_SPACING;
        let spacing_after = NghiDinh30::HEADER_SPACING_AFTER;
        
        format!(
            r#"
        <w:tbl>
            <w:tblPr>
                <w:tblW w:w="9576" w:type="dxa"/>
                <w:tblBorders>
                    <w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                    <w:left w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                    <w:bottom w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                    <w:right w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                    <w:insideH w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                    <w:insideV w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                </w:tblBorders>
            </w:tblPr>
            <w:tblGrid>
                <w:gridCol w:w="4788"/>
                <w:gridCol w:w="4788"/>
            </w:tblGrid>
            <w:tr>
                <w:tc>
                    <w:tcPr><w:tcW w:w="4788" w:type="dxa"/></w:tcPr>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:b/>
                                <w:u w:val="single"/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">{}</w:t>
                        </w:r>
                    </w:p>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:b/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">Mã đề thi: {}</w:t>
                        </w:r>
                    </w:p>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:b/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">ĐỀ CHÍNH THỨC</w:t>
                        </w:r>
                    </w:p>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">(Gồm </w:t>
                        </w:r>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:b/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">{}</w:t>
                        </w:r>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve"> trang)</w:t>
                        </w:r>
                    </w:p>
                </w:tc>
                <w:tc>
                    <w:tcPr><w:tcW w:w="4788" w:type="dxa"/></w:tcPr>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:b/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">{}</w:t>
                        </w:r>
                    </w:p>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:b/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">Năm học: {}</w:t>
                        </w:r>
                    </w:p>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:b/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">Tên môn: {}, {}</w:t>
                        </w:r>
                    </w:p>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:i/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">Thời gian làm bài: {}</w:t>
                        </w:r>
                    </w:p>
                    <w:p>
                        <w:pPr>
                            <w:jc w:val="center"/>
                            <w:spacing w:line="{}" w:lineRule="auto" w:after="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:rPr>
                                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                                <w:i/>
                                <w:sz w:val="{}"/>
                            </w:rPr>
                            <w:t xml:space="preserve">(Không kể thời gian phát đề)</w:t>
                        </w:r>
                    </w:p>
                </w:tc>
            </w:tr>
        </w:tbl>
        <w:p/>
"#,
            // Left column - School name (bold + underline)
            line_spacing, spacing_after, font, font, font, font, size, self.school_name,
            // Exam code (bold)
            line_spacing, spacing_after, font, font, font, font, size, self.exam_code,
            // "ĐỀ CHÍNH THỨC" (bold)
            line_spacing, spacing_after, font, font, font, font, size,
            // Page count ("Gồm" normal, number bold, "trang" normal)
            line_spacing, spacing_after, font, font, font, font, size,
            font, font, font, font, size, page_text,
            font, font, font, font, size,
            // Right column - Exam name (bold)
            line_spacing, spacing_after, font, font, font, font, size, self.exam_name,
            // Academic year (bold)
            line_spacing, spacing_after, font, font, font, font, size, self.academic_year,
            // Subject and grade (bold)
            line_spacing, spacing_after, font, font, font, font, size, self.subject, self.grade,
            // Duration (italic)
            line_spacing, spacing_after, font, font, font, font, size, self.duration_minutes,
            // Distribution note (italic)
            line_spacing, spacing_after, font, font, font, font, size
        )
    }

    /// Generate XML for a single question
    fn generate_question_xml(&self, num: usize, question: &Question, image_map: &HashMap<String, ImageInfo>) -> String {
        let mut xml = String::new();

        // Question stem paragraph
        xml.push_str("<w:p>");
        
        // Check if first segment already contains "Câu X." prefix
        let stem_has_prefix = question.stem.first().map_or(false, |seg| {
            match seg {
                Segment::Text { text, .. } => text.contains("Câu"),
                _ => false,
            }
        });

        if !stem_has_prefix {
            // Add question number prefix if not already in content
            xml.push_str(&format!(
                r#"<w:r><w:rPr><w:b/><w:sz w:val="{}"/><w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/></w:rPr><w:t>Câu {}. </w:t></w:r>"#,
                NghiDinh30::FONT_SIZE_BODY,
                NghiDinh30::FONT_NAME,
                NghiDinh30::FONT_NAME,
                NghiDinh30::FONT_NAME,
                NghiDinh30::FONT_NAME,
                num
            ));
        }

        // Stem content
        for segment in &question.stem {
            xml.push_str(&self.segment_to_xml(segment, image_map));
        }
        xml.push_str("</w:p>");

        // Options
        for option in &question.options {
            xml.push_str("<w:p>");
            xml.push_str(
                r#"<w:pPr><w:ind w:left="720"/></w:pPr>"#, // 0.5" indent
            );
            
            // Check if first segment already contains option label
            let option_has_prefix = option.content.first().map_or(false, |seg| {
                match seg {
                    Segment::Text { text, .. } => {
                        text.starts_with(&format!("{}.", option.label)) ||
                        text.starts_with(&format!("#{}.", option.label)) ||
                        text.contains(&format!("{}. ", option.label)) ||
                        text.contains(&format!("#{}. ", option.label))
                    },
                    _ => false,
                }
            });

            if !option_has_prefix {
                // Add option label prefix if not already in content
                let label_str = if option.locked {
                    format!("#{}. ", option.label)
                } else {
                    format!("{}. ", option.label)
                };
                xml.push_str(&format!(
                    r#"<w:r><w:rPr><w:b/><w:sz w:val="{}"/><w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/></w:rPr><w:t>{}</w:t></w:r>"#,
                    NghiDinh30::FONT_SIZE_BODY,
                    NghiDinh30::FONT_NAME,
                    NghiDinh30::FONT_NAME,
                    NghiDinh30::FONT_NAME,
                    NghiDinh30::FONT_NAME,
                    label_str
                ));
            }

            // Option content
            for segment in &option.content {
                xml.push_str(&self.segment_to_xml(segment, image_map));
            }
            xml.push_str("</w:p>");
        }

        // Spacer
        xml.push_str("<w:p/>");

        xml
    }

    /// Convert segment to OpenXML - generate clean XML with embedded images
    fn segment_to_xml(&self, segment: &Segment, image_map: &HashMap<String, ImageInfo>) -> String {
        match segment {
            Segment::Text { text, .. } => {
                // Generate clean text run with basic formatting
                if text.is_empty() {
                    return String::new();
                }
                
                // Escape XML special characters
                let escaped = text
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;");
                
                format!(
                    r#"<w:r><w:rPr><w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/><w:sz w:val="{}"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
                    NghiDinh30::FONT_NAME,
                    NghiDinh30::FONT_NAME,
                    NghiDinh30::FONT_NAME,
                    NghiDinh30::FONT_NAME,
                    NghiDinh30::FONT_SIZE_BODY,
                    escaped
                )
            }
            Segment::Image { asset_path, width_emu, height_emu, .. } => {
                // Use dimensions from original document if available, otherwise use from file
                if *width_emu > 0 && *height_emu > 0 {
                    // Use original document dimensions
                    if let Some(img_info) = image_map.get(asset_path) {
                        self.generate_image_xml(&img_info.rel_id, *width_emu, *height_emu)
                    } else {
                        format!(r#"<w:r><w:rPr><w:sz w:val="{}"/></w:rPr><w:t>[Image not found]</w:t></w:r>"#, NghiDinh30::FONT_SIZE_BODY)
                    }
                } else {
                    // Fallback to image file dimensions
                    if let Some(img_info) = image_map.get(asset_path) {
                        self.generate_image_xml(&img_info.rel_id, img_info.width_emu, img_info.height_emu)
                    } else {
                        format!(r#"<w:r><w:rPr><w:sz w:val="{}"/></w:rPr><w:t>[Image not found]</w:t></w:r>"#, NghiDinh30::FONT_SIZE_BODY)
                    }
                }
            }
            Segment::Math { omml, .. } => {
                // Use the OMML content directly
                // Wrap it in a run
                format!(r#"<w:r>{}</w:r>"#, omml)
            }
        }
    }

    /// Generate DrawingML XML for an image with actual dimensions
    fn generate_image_xml(&self, rel_id: &str, width_emu: i64, height_emu: i64) -> String {

        format!(
            r#"<w:r>
    <w:drawing>
        <wp:inline distT="0" distB="0" distL="0" distR="0">
            <wp:extent cx="{}" cy="{}"/>
            <wp:effectExtent l="0" t="0" r="0" b="0"/>
            <wp:docPr id="1" name="Picture"/>
            <wp:cNvGraphicFramePr>
                <a:graphicFrameLocks xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" noChangeAspect="1"/>
            </wp:cNvGraphicFramePr>
            <a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
                <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">
                    <pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
                        <pic:nvPicPr>
                            <pic:cNvPr id="0" name="Picture"/>
                            <pic:cNvPicPr/>
                        </pic:nvPicPr>
                        <pic:blipFill>
                            <a:blip r:embed="{}"/>
                            <a:stretch>
                                <a:fillRect/>
                            </a:stretch>
                        </pic:blipFill>
                        <pic:spPr>
                            <a:xfrm>
                                <a:off x="0" y="0"/>
                                <a:ext cx="{}" cy="{}"/>
                            </a:xfrm>
                            <a:prstGeom prst="rect">
                                <a:avLst/>
                            </a:prstGeom>
                        </pic:spPr>
                    </pic:pic>
                </a:graphicData>
            </a:graphic>
        </wp:inline>
    </w:drawing>
</w:r>"#,
            width_emu, height_emu, rel_id, width_emu, height_emu
        )
    }

    /// Generate word/_rels/document.xml.rels with image relationships
    fn generate_document_rels(&self, image_map: &HashMap<String, ImageInfo>) -> String {
        let mut rels = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rIdFooter1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>"#,
        );

        // Add image relationships
        for (_, img_info) in image_map {
            let filename = img_info.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("image.png");

            rels.push_str(&format!(
                r#"
    <Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/{}"/>"#,
                img_info.rel_id, filename
            ));
        }

        rels.push_str("\n</Relationships>");
        rels
    }

    /// Generate word/styles.xml
    fn generate_styles_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:docDefaults>
        <w:rPrDefault>
            <w:rPr>
                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                <w:sz w:val="{}"/>
            </w:rPr>
        </w:rPrDefault>
    </w:docDefaults>
</w:styles>"#,
            NghiDinh30::FONT_NAME,
            NghiDinh30::FONT_NAME,
            NghiDinh30::FONT_NAME,
            NghiDinh30::FONT_NAME,
            NghiDinh30::FONT_SIZE_BODY
        )
    }

    /// Generate word/footer1.xml with page numbers (Nghị định 30)
    /// Page numbers: size 13, centered, not shown on first page
    fn generate_footer_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:p>
        <w:pPr>
            <w:jc w:val="center"/>
        </w:pPr>
        <w:r>
            <w:rPr>
                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                <w:sz w:val="{}"/>
            </w:rPr>
            <w:fldChar w:fldCharType="begin"/>
        </w:r>
        <w:r>
            <w:rPr>
                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                <w:sz w:val="{}"/>
            </w:rPr>
            <w:instrText xml:space="preserve"> PAGE </w:instrText>
        </w:r>
        <w:r>
            <w:rPr>
                <w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}" w:eastAsia="{}"/>
                <w:sz w:val="{}"/>
            </w:rPr>
            <w:fldChar w:fldCharType="end"/>
        </w:r>
    </w:p>
</w:ftr>"#,
            NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_SIZE_PAGE_NUMBER,
            NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_SIZE_PAGE_NUMBER,
            NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_NAME, NghiDinh30::FONT_SIZE_PAGE_NUMBER
        )
    }

    /// Generate section properties (Nghị định 30)
    /// A4 paper (210mm x 297mm)
    /// Margins: top/bottom 20mm, left 30mm, right 15mm
    fn generate_section_properties(&self) -> String {
        format!(
            r#"
        <w:sectPr>
            <w:footerReference w:type="default" r:id="rIdFooter1"/>
            <w:pgSz w:w="{}" w:h="{}"/>
            <w:pgMar w:top="{}" w:right="{}" w:bottom="{}" w:left="{}" w:header="{}" w:footer="{}" w:gutter="0"/>
            <w:cols w:space="708"/>
            <w:titlePg/>
        </w:sectPr>"#,
            NghiDinh30::PAGE_WIDTH_TWIPS,
            NghiDinh30::PAGE_HEIGHT_TWIPS,
            NghiDinh30::MARGIN_TOP_TWIPS,
            NghiDinh30::MARGIN_RIGHT_TWIPS,
            NghiDinh30::MARGIN_BOTTOM_TWIPS,
            NghiDinh30::MARGIN_LEFT_TWIPS,
            NghiDinh30::MARGIN_HEADER_TWIPS,
            NghiDinh30::MARGIN_FOOTER_TWIPS
        )
    }

    /// Embed images into DOCX
    fn embed_images(
        &self,
        zip: &mut ZipWriter<BufWriter<File>>,
        options: FileOptions,
        image_map: &HashMap<String, ImageInfo>,
    ) -> Result<(), std::io::Error> {
        use std::io::Read;

        for (_, img_info) in image_map {
            // Read image file
            let mut file = File::open(&img_info.path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            // Get filename
            let filename = img_info.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("image.png");

            // Write to word/media/ directory in zip
            let media_path = format!("word/media/{}", filename);
            zip.start_file(&media_path, options)?;
            zip.write_all(&buffer)?;
        }

        Ok(())
    }
}
