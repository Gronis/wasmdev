
use std::io;
use std::fs;
use std::path::{Path, PathBuf};

/// Recusivly list files 3-layers down.
pub fn list_files_recursively<P: AsRef<Path>>(path: P) -> io::Result<Vec<PathBuf>> {
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
