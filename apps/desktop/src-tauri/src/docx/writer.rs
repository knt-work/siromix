// src-tauri/src/docx/writer.rs
//! DOCX writer with manual OMML injection
//! Generates valid OpenXML .docx files with proper formatting

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

use super::model::{Question, Segment};

/// Exam writer that generates a complete DOCX file
pub struct ExamWriter {
    pub exam_code: String,
    pub questions: Vec<Question>,
    pub exam_title: String,
    pub subject: String,
    pub duration_minutes: u32,
    pub assets_dir: PathBuf,
}

impl ExamWriter {
    /// Write DOCX file to disk
    pub fn write_to_file(&self, output_path: &Path) -> Result<(), std::io::Error> {
        let file = File::create(output_path)?;
        let mut zip = ZipWriter::new(BufWriter::new(file));
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // 1. [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(self.generate_content_types().as_bytes())?;

        // 2. _rels/.rels
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(self.generate_root_rels().as_bytes())?;

        // 3. word/document.xml (main content)
        zip.start_file("word/document.xml", options)?;
        zip.write_all(self.generate_document_xml().as_bytes())?;

        // 4. word/_rels/document.xml.rels
        zip.start_file("word/_rels/document.xml.rels", options)?;
        zip.write_all(self.generate_document_rels().as_bytes())?;

        // 5. word/styles.xml
        zip.start_file("word/styles.xml", options)?;
        zip.write_all(self.generate_styles_xml().as_bytes())?;

        // 6. Embed images
        self.embed_images(&mut zip, options)?;

        zip.finish()?;
        Ok(())
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
    fn generate_document_xml(&self) -> String {
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
            doc.push_str(&self.generate_question_xml(idx + 1, question));
        }

        doc.push_str(
            r#"
    </w:body>
</w:document>"#,
        );
        doc
    }

    /// Generate header section
    fn generate_header(&self) -> String {
        format!(
            r#"
        <w:p>
            <w:pPr><w:jc w:val="center"/></w:pPr>
            <w:r>
                <w:rPr>
                    <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/>
                    <w:b/>
                    <w:sz w:val="28"/>
                </w:rPr>
                <w:t>{}</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:pPr><w:jc w:val="center"/></w:pPr>
            <w:r>
                <w:rPr>
                    <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/>
                    <w:b/>
                    <w:sz w:val="32"/>
                </w:rPr>
                <w:t>{} - Mã đề: {}</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:pPr><w:jc w:val="center"/></w:pPr>
            <w:r>
                <w:rPr>
                    <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/>
                    <w:sz w:val="24"/>
                </w:rPr>
                <w:t>Môn: {} - Thời gian: {} phút</w:t>
            </w:r>
        </w:p>
        <w:p/>
"#,
            self.exam_title, self.exam_title, self.exam_code, self.subject, self.duration_minutes
        )
    }

    /// Generate XML for a single question
    fn generate_question_xml(&self, num: usize, question: &Question) -> String {
        let mut xml = String::new();

        // Question stem paragraph
        xml.push_str("<w:p>");
        xml.push_str(&format!(
            r#"<w:r><w:rPr><w:b/><w:sz w:val="26"/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>Câu {}. </w:t></w:r>"#,
            num
        ));

        // Stem content
        for segment in &question.stem {
            xml.push_str(&self.segment_to_xml(segment));
        }
        xml.push_str("</w:p>");

        // Options
        for option in &question.options {
            xml.push_str("<w:p>");
            xml.push_str(
                r#"<w:pPr><w:ind w:left="720"/></w:pPr>"#, // 0.5" indent
            );
            xml.push_str(&format!(
                r#"<w:r><w:rPr><w:b/><w:sz w:val="26"/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}. </w:t></w:r>"#,
                option.label
            ));

            for segment in &option.content {
                xml.push_str(&self.segment_to_xml(segment));
            }
            xml.push_str("</w:p>");
        }

        // Spacer
        xml.push_str("<w:p/>");

        xml
    }

    /// Convert segment to OpenXML
    fn segment_to_xml(&self, segment: &Segment) -> String {
        match segment {
            Segment::Text { text } => {
                let escaped = text
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                format!(
                    r#"<w:r><w:rPr><w:sz w:val="26"/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
                    escaped
                )
            }
            Segment::Image { asset_path, .. } => {
                // Will be replaced with actual image embed logic
                format!(r#"<w:r><w:t>[Image: {}]</w:t></w:r>"#, asset_path)
            }
            Segment::Math { omml } => {
                // 🔥 Direct OMML injection
                format!(
                    r#"<w:r><m:oMathPara><m:oMath>{}</m:oMath></m:oMathPara></w:r>"#,
                    omml
                )
            }
        }
    }

    /// Generate word/_rels/document.xml.rels
    fn generate_document_rels(&self) -> String {
        let mut rels = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );

        // Add image relationships (will be populated later)
        // For now, just close
        rels.push_str("\n</Relationships>");
        rels
    }

    /// Generate word/styles.xml
    fn generate_styles_xml(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:docDefaults>
        <w:rPrDefault>
            <w:rPr>
                <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/>
                <w:sz w:val="26"/>
            </w:rPr>
        </w:rPrDefault>
    </w:docDefaults>
</w:styles>"#
            .to_string()
    }

    /// Embed images into DOCX
    fn embed_images(
        &self,
        _zip: &mut ZipWriter<BufWriter<File>>,
        _options: FileOptions,
    ) -> Result<(), std::io::Error> {
        // TODO: Copy images from assets_dir to word/media/
        // TODO: Update document.xml.rels
        Ok(())
    }
}
