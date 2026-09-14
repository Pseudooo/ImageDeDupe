use std::{env, fs};
use std::ffi::OsStr;
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
        } else if path.is_file() && is_valid_extension(path.extension()) {
            files.push(path);
        }
    }

    return files;
}

fn is_valid_extension(extension: Option<&OsStr>) -> bool {
    let valid_extensions = vec!["jpg", "jpeg", "png", "heic"];

    if let Some(e) = extension {
        return valid_extensions.contains(&e.to_ascii_lowercase().to_str().unwrap());
    }

    return false;
}
