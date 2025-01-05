use crate::helpers::fs_helpers::check_is_mod_dir;
use std::fs::{DirEntry, ReadDir};

pub fn get_mod_dirs(dir: ReadDir) -> Vec<DirEntry> {
    dir.filter_map(|e| match e {
        Ok(e) => Some(e),
        Err(e) => {
            eprintln!("Failed to read entry: {}", e);
            None
        }
    })
    .filter(|e| check_is_mod_dir(&e.path()))
    .collect()
}
