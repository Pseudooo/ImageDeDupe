use indicatif::ProgressBar;
use std::ffi::OsStr;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn get_images_from_target_directory(target: &PathBuf, progress_bar: Option<&ProgressBar>) -> Result<Vec<PathBuf>, String> {
    let mut found_images: Vec<PathBuf> = Vec::new();

    for entry_result in WalkDir::new(target).into_iter() {
        let entry = entry_result.map_err(|e| format!("Failed to process directory entry: {}", e))?;

        if !entry.file_type().is_file() {
            continue;
        }

        if let Some(progress_bar) = progress_bar {
            progress_bar.inc(1);
        }

        if is_valid_extension(entry.path().extension()) {
            progress_bar.map(|p| p.set_message(found_images.len().to_string()));
            found_images.push(entry.into_path());
        }
    }

    Ok(found_images)
}

fn is_valid_extension(extension: Option<&OsStr>) -> bool {
    let valid_extensions = ["jpg", "jpeg", "png", "heic"];

    if let Some(e) = extension {
        return valid_extensions.contains(&e.to_ascii_lowercase().to_str().unwrap());
    }

    false
}
