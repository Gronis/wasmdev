use std::{fs, path::Path};

/// Removes all directories in this directory that has no children
pub fn remove_empty_dirs(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                remove_empty_dirs(&path)?;
            } 
        }
        // We have to read dir again after potiential sub-dirs was removed
        if fs::read_dir(path)?.count() == 0 {
            fs::remove_dir(path)?;
        }
    }
    Ok(())
}