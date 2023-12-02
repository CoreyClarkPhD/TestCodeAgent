use std::fs;

fn get_cpp_files_in_directory(directory_path: &str) -> Vec<String> {
    let entries = fs::read_dir(directory_path)
        .expect("Failed to read directory")
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" {
                        return Some(path.to_str().unwrap().to_string());
                    }
                }
            }
            None
        })
        .collect();

    entries
}

fn main() {
    let directory_path = "./job-system-lib/";
    cc::Build::new()
        .files(get_cpp_files_in_directory(directory_path).as_slice())
        .std("c++17")
        .cpp(true)
        .compile("libjobsystem");
}
