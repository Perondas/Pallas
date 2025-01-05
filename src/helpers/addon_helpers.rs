use std::path::{Path, PathBuf};
use crate::helpers::fs_helpers::get_path_extension;
use anyhow::Context;
use anyhow::Result;

/// Get all PBOs in a directory
pub fn get_pbos_in_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let entries =
        std::fs::read_dir(path).context("Failed to read the mod directory")?;

    let addons = entries.filter_map(|e| match e {
        Ok(e) => match get_path_extension(&e.path()) {
            Some("pbo") => Some(e),
            Some(_) => None,
            None => None,
        },
        Err(e) => {
            eprintln!("Failed to read entry: {}", e);
            None
        }
    });

    Ok(addons.map(|e| e.path()).collect())
}

pub fn check_contains_ebo(path: &Path) -> bool {
    let mut entries = std::fs::read_dir(path).expect("Failed to read the mod directory");

    entries.any(|e| match e {
        Ok(e) => matches!(get_path_extension(&e.path()), Some("ebo")),
        Err(_) => { false}
    })
}