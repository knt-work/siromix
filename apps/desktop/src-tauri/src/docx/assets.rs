use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};

use zip::read::ZipArchive;

use super::{AppError, ExtractedAsset};

/// Extract all image files under `word/media/` from a `.docx` into
/// the given `assets_dir`. Returns the list of extracted assets
/// (file name and absolute path).
pub fn extract_media(
    docx_path: &Path,
    assets_dir: &Path,
) -> Result<Vec<ExtractedAsset>, AppError> {
    // Ensure the destination directory exists
    fs::create_dir_all(assets_dir)?;

    let file = File::open(docx_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut extracted = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        // Skip directories
        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();

        // Only consider files under `word/media/`
        if !name.starts_with("word/media/") {
            continue;
        }

        // Only extract common image types
        let is_image = Path::new(&name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                let ext = ext.to_ascii_lowercase();
                matches!(
                    ext.as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tif" | "tiff" | "emf" | "wmf"
                )
            })
            .unwrap_or(false);

        if !is_image {
            continue;
        }

        let file_name = Path::new(&name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("media")
            .to_string();

        let out_path: PathBuf = assets_dir.join(&file_name);

        // Write the media file out
        let mut out_file = File::create(&out_path)?;
        copy(&mut entry, &mut out_file)?;

        // Best-effort absolute path; if canonicalize fails, keep as-is
        let absolute_path = out_path
            .canonicalize()
            .unwrap_or(out_path.clone());

        extracted.push(ExtractedAsset {
            file_name,
            absolute_path,
        });
    }

    Ok(extracted)
}
