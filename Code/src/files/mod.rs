use anyhow::Result;
use std::path::PathBuf;

pub fn get_all_cpp_files_in_folder_path(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_dir() {
        for entry in std::fs::read_dir(path).expect("Read dir") {
            let entry = entry.expect("Entry");
            let path = entry.path();
            if path.is_dir() {
                files.append(&mut get_all_cpp_files_in_folder_path(&path)?);
            } else {
                if path.extension().unwrap_or_default() == "cpp" {
                    files.push(path);
                }
            }
        }
    } else {
        if path.extension().unwrap_or_default() == "cpp" {
            files.push(path.to_owned());
        }
    }

    return Ok(files);
}
