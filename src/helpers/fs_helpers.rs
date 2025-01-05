use anyhow::anyhow;
use anyhow::Result;
use std::borrow::Cow;
use std::path::Path;

/// Check if a path is a directory and starts with '@'
pub fn check_is_mod_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    let name = match path.file_name() {
        Some(name) => name,
        None => return false,
    };

    match name.to_str() {
        Some(name) => name.starts_with('@'),
        None => false,
    }
}

pub fn get_path_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(|ext| ext.to_str())
}

pub fn get_folder_name(path: &Path) -> Result<Cow<'_, str>> {
    Ok(path
        .file_name()
        .ok_or(anyhow!("Failed to get folder name"))?
        .to_string_lossy())
}
