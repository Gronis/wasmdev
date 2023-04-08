
use std::io;
use std::fs;
use std::path::{Path, PathBuf};

/// Recusivly list files 3-layers down.
pub fn list_files_recursively(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let mut files = vec![];
    let mut paths = vec![path.as_ref().to_path_buf()];
    let mut traverse = |paths_in: &mut Vec<PathBuf>| -> io::Result<Vec<PathBuf>> {
        let mut paths_out = vec![];
        for path in paths_in.drain(..) {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    paths_out.push(path);
                } else {
                    files.push(path);
                }
            }
        }
        Ok(paths_out)
    };

    // Recurse 3 layers down
    let mut paths = traverse(&mut paths)?;
    let mut paths = traverse(&mut paths)?;
    let _         = traverse(&mut paths)?;

    Ok(files)
}

/// Removes all directories in this directory that has no children
pub fn remove_empty_dirs(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                remove_empty_dirs(path)?;
            } 
        }
        // We have to read dir again after potiential sub-dirs was removed
        if fs::read_dir(path)?.count() == 0 {
            fs::remove_dir(path)?;
        }
    }
    Ok(())
}

/// Same as fs::create_dir_all, but only up to the parent
pub fn create_parent_dir_all(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    let path = path.parent().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "Unable to get parent directory"))?;
    fs::create_dir_all(path)
}