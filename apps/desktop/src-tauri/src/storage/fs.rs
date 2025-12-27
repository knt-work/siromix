use std::fs;
use std::path::Path;

pub fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("Không tạo được thư mục {}: {}", path.display(), e))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Không tạo được thư mục cha {}: {}", parent.display(), e))?;
    }
    Ok(())
}

pub fn copy_file(src: &Path, dest: &Path) -> Result<(), String> {
    if !src.exists() {
        return Err(format!("File nguồn không tồn tại: {}", src.display()));
    }

    ensure_parent_dir(dest)?;

    fs::copy(src, dest).map(|_| ()).map_err(|e| {
        format!(
            "Không copy được file từ {} tới {}: {}",
            src.display(),
            dest.display(),
            e
        )
    })
}
