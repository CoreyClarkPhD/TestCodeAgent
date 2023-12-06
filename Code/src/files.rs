use anyhow::Result;
use std::{io::Write, path::PathBuf};

pub fn get_all_cpp_files_in_folder_path(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_dir() {
        for entry in std::fs::read_dir(path).expect("Read dir") {
            let entry = entry.expect("Entry");
            let path = entry.path();
            if path.is_dir() {
                files.append(&mut get_all_cpp_files_in_folder_path(&path)?);
            } else if path.extension().unwrap_or_default() == "cpp" {
                files.push(path);
            }
        }
    } else if path.extension().unwrap_or_default() == "cpp" {
        files.push(path.to_owned());
    }

    Ok(files)
}


pub fn replace_code(path: &PathBuf, new_code: String) {
    // Smartly insert the new code
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .expect("Open file");

    // Match the first and last lines
    let new_code_lines: Vec<&str> = new_code.lines().collect();

    let new_first_line = new_code_lines
        .iter()
        .filter(|line| !line.is_empty())
        .next()
        .unwrap();
    let new_last_line = new_code_lines
        .iter()
        .rev()
        .filter(|line| !line.is_empty())
        .next()
        .unwrap();

    let mut start_index = 0;
    let mut end_index = 0;

    let mut old_code_lines: Vec<String> = std::fs::read_to_string(path)
        .expect("Read file")
        .lines()
        .map(|line| line.to_owned())
        .collect();

    for (i, line) in old_code_lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        if line == new_first_line {
            start_index = i;
            continue;
        }
        if line == new_last_line {
            end_index = i;
            break;
        }
    }

    // If still not found, just replace the whole file
    if start_index == 0 && end_index == 0 {
        file.write_all(new_code.as_bytes()).expect("Write file");
        return;
    }

    // Remove the range and replace with the new code
    old_code_lines.splice(
        start_index..end_index + 1,
        new_code_lines.iter().map(|s| s.to_string()),
    );

    // Write the new code
    file.write_all(old_code_lines.join("\n").as_bytes())
        .expect("Write file");
}
