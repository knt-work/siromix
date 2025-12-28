use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use std::process::Command;

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
            converted_path: None,
        });
    }

    // Post-process: attempt to convert WMF/EMF files to PNG
    convert_wmf_assets(&mut extracted, assets_dir);

    Ok(extracted)
}

/// Attempt to convert WMF/EMF files to PNG using ImageMagick.
/// 
/// This function tries to use the system's ImageMagick `magick` command
/// to convert WMF/EMF files to PNG. If conversion succeeds, the
/// `converted_path` field is populated.
/// 
/// Falls back gracefully if ImageMagick is not available.
fn convert_wmf_assets(assets: &mut Vec<ExtractedAsset>, assets_dir: &Path) {
    for asset in assets.iter_mut() {
        // Check if this is a WMF or EMF file
        let ext = asset.absolute_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        
        let is_wmf = matches!(ext.as_deref(), Some("wmf") | Some("emf"));
        if !is_wmf {
            continue;
        }

        // Generate output PNG path
        let png_filename = asset.file_name
            .trim_end_matches(".wmf")
            .trim_end_matches(".emf")
            .trim_end_matches(".WMF")
            .trim_end_matches(".EMF")
            .to_string() + ".png";
        
        let png_path = assets_dir.join(&png_filename);

        // Try to convert using ImageMagick
        match convert_wmf_to_png(&asset.absolute_path, &png_path) {
            Ok(true) => {
                println!("[WMF] Successfully converted: {} → {}", 
                    asset.file_name, png_filename);
                asset.converted_path = Some(png_path);
            }
            Ok(false) => {
                println!("[WMF] ImageMagick not available, keeping original: {}", 
                    asset.file_name);
            }
            Err(e) => {
                eprintln!("[WMF] Conversion failed for {}: {:?}", asset.file_name, e);
            }
        }
    }
}

/// Try to convert a WMF/EMF file to PNG using ImageMagick.
/// 
/// Returns:
/// - Ok(true) if conversion succeeded
/// - Ok(false) if ImageMagick is not available
/// - Err(_) if conversion was attempted but failed
fn convert_wmf_to_png(wmf_path: &Path, png_path: &Path) -> Result<bool, std::io::Error> {
    // Convert paths to strings (ImageMagick needs string args)
    let wmf_str = wmf_path.to_string_lossy();
    let png_str = png_path.to_string_lossy();
    
    // Simple conversion without resize - let frontend handle sizing via CSS
    let output = Command::new("magick")
        .arg(wmf_str.as_ref())
        .arg("-density")
        .arg("96")  // Screen resolution
        .arg("-trim")  // Remove whitespace
        .arg(png_str.as_ref())
        .output();

    match output {
        Ok(result) if result.status.success() => {
            println!("[WMF] Successfully converted: {}", wmf_path.file_name().unwrap_or_default().to_string_lossy());
            Ok(true)
        }
        Ok(result) => {
            // Command ran but failed
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stdout = String::from_utf8_lossy(&result.stdout);
            eprintln!("[WMF] ImageMagick error: {}{}", stderr, stdout);
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("ImageMagick failed: {}{}", stderr, stdout)
            ))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // ImageMagick not available
            Ok(false)
        }
        Err(e) => Err(e),
    }
}
