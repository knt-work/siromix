use std::fs::File;
use std::io::Read;
use std::path::Path;

use zip::read::ZipArchive;

use super::AppError;

/// Open a .docx file as a ZIP archive and read the `word/document.xml`
/// entry into a UTF-8 string.
pub fn read_document_xml(docx_path: &Path) -> Result<String, AppError> {
    // Open the .docx file as a regular file first
    let file = File::open(docx_path)?;

    // Treat it as a ZIP archive
    let mut archive = ZipArchive::new(file)?;

    // Access the `word/document.xml` entry
    let mut doc_xml = archive.by_name("word/document.xml")?;

    // Read the entry contents into memory
    let mut buffer = Vec::new();
    doc_xml.read_to_end(&mut buffer)?;

    // Convert bytes to UTF-8 string
    let xml = String::from_utf8(buffer)?;

    Ok(xml)
}
