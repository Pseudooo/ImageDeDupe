use std::{env, fs};
use std::path::PathBuf;

fn main() {
    println!("Finding files in current directory...");

    let curr_path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => panic!("Failed to read current directory: {}", e),
    };
    let files = get_files_from_path(curr_path);

    println!("Found {} files", files.len());
}

fn get_files_from_path(path: PathBuf) -> Vec<PathBuf> {
    let dir_entries = match fs::read_dir(&path) {
        Ok(dir_entries) => dir_entries,
        Err(error) => panic!("failed to read directory: {}", error),
    };

    let mut files: Vec<PathBuf> = Vec::new();

    for entry in dir_entries {
        let path = entry.unwrap().path();

        if path.is_dir() {
            files.append(&mut get_files_from_path(path));
        } else if path.is_file() {
            files.push(path);
        }
    }

    return files;
}
