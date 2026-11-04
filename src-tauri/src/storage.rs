use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

pub fn photos_dir(app_data: &Path) -> PathBuf {
    app_data.join("photos")
}

/// Build the `asset://` URL the WebView uses to load a stored photo.
/// Filenames are UUID + lowercased extension, so no URL-encoding is required.
pub fn asset_url(app_data: &Path, filename: &str) -> String {
    let abs = photos_dir(app_data).join(filename);
    format!("asset://localhost/{}", abs.to_string_lossy().replace('\\', "/"))
}

/// Copy a user-picked file into `<app-data>/photos/`, returning the new filename.
pub fn save_photo_file(app_data: &Path, source: &Path) -> AppResult<String> {
    if !source.is_file() {
        return Err(AppError::InvalidInput(format!("not a file: {}", source.display())));
    }
    let dir = photos_dir(app_data);
    std::fs::create_dir_all(&dir)?;
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_ascii_lowercase();
    let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    std::fs::copy(source, dir.join(&filename))?;
    Ok(filename)
}

pub fn delete_photo_file(app_data: &Path, filename: &str) -> AppResult<()> {
    let path = photos_dir(app_data).join(filename);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Copy a stored photo out to a user-chosen destination (export).
pub fn copy_photo_out(app_data: &Path, filename: &str, dest: &Path) -> AppResult<()> {
    let src = photos_dir(app_data).join(filename);
    if !src.is_file() {
        return Err(AppError::NotFound(filename.to_string()));
    }
    std::fs::copy(src, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_app_data() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lovelygarden-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_bytes(path: &Path, bytes: &[u8]) {
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn save_copies_file_into_photos_dir() {
        let app = temp_app_data();
        let src = app.join("in.jpg");
        write_bytes(&src, b"fake-image-bytes");
        let name = save_photo_file(&app, &src).unwrap();
        assert!(name.ends_with(".jpg"));
        assert!(photos_dir(&app).join(&name).exists());
        let _ = std::fs::remove_dir_all(&app);
    }

    #[test]
    fn asset_url_uses_asset_protocol() {
        let app = temp_app_data();
        let url = asset_url(&app, "abc.jpg");
        assert!(url.starts_with("asset://localhost/"));
        assert!(url.ends_with("photos/abc.jpg"));
        let _ = std::fs::remove_dir_all(&app);
    }

    #[test]
    fn delete_removes_file() {
        let app = temp_app_data();
        let src = app.join("in.png");
        write_bytes(&src, b"x");
        let name = save_photo_file(&app, &src).unwrap();
        assert!(photos_dir(&app).join(&name).exists());
        delete_photo_file(&app, &name).unwrap();
        assert!(!photos_dir(&app).join(&name).exists());
        let _ = std::fs::remove_dir_all(&app);
    }

    #[test]
    fn copy_out_writes_dest() {
        let app = temp_app_data();
        let src = app.join("in.png");
        write_bytes(&src, b"x");
        let name = save_photo_file(&app, &src).unwrap();
        let dest = app.join("out.png");
        copy_photo_out(&app, &name, &dest).unwrap();
        assert!(dest.exists());
        let _ = std::fs::remove_dir_all(&app);
    }

    #[test]
    fn rejects_missing_source() {
        let app = temp_app_data();
        assert!(save_photo_file(&app, &app.join("nope.jpg")).is_err());
        let _ = std::fs::remove_dir_all(&app);
    }
}
