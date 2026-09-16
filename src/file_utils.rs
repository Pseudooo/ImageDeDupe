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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use tempfile::tempdir;

    /// Given a known directory and files exist:
    /// ```
    /// ./target_dir/
    /// ./target_dir/file1.jpg
    /// ./target_dir/file2.jpeg
    /// ./target_dir/file3.png
    /// ./target_dir/file4.heic
    /// ```
    /// Then when scanning `target_dir` all contained files should be discovered
    #[test]
    fn given_directory_with_valid_files_when_scan_then_find_files() {
        // Arrange
        let tmp_dir = tempdir().expect("Failed to create temp directory");
        let tmp_path = tmp_dir.path();

        let file1 = tmp_path.join("file1.jpg");
        let file2 = tmp_path.join("file2.jpeg");
        let file3 = tmp_path.join("file3.png");
        let file4 = tmp_path.join("file4.heic");

        File::create(&file1).expect("Failed to create file1");
        File::create(&file2).expect("Failed to create file2");
        File::create(&file3).expect("Failed to create file3");
        File::create(&file4).expect("Failed to create file4");

        // Act
        let target_path = tmp_path.to_path_buf();
        let result = get_images_from_target_directory(&target_path, None);

        // Assert
        assert!(result.is_ok(), "Scan should succeed");

        let found_images = result.unwrap();
        assert_eq!(found_images.len(), 4, "Should find exactly 4 images");
        assert!(found_images.contains(&file1), "Should discover file1.jpg");
        assert!(found_images.contains(&file2), "Should discover file2.jpeg");
        assert!(found_images.contains(&file3), "Should discover file3.png");
        assert!(found_images.contains(&file4), "Should discover file4.heic");
    }

    /// Given a known directory structure with files inside nested directories like so:
    /// ```
    /// ./target_dir/
    /// ./target_dir/file1.png
    /// ./target_dir/nested_dir/file2.png
    /// ```
    /// When scanning `target_dir` then both files are discovered
    #[test]
    fn given_nested_directory_structure_with_valid_files_when_scan_then_find_all_files() {
        // Arrange
        let tmp_dir = tempdir().expect("Failed to create temp directory");
        let tmp_path = tmp_dir.path();

        let file1 = tmp_path.join("file1.jpg");
        File::create(&file1).expect("Failed to create file1");

        let nested_dir_path = tmp_path.join("nested_dir");
        fs::create_dir(&nested_dir_path).expect("Failed to create nested_dir");
        let file2 = nested_dir_path.join("file2.jpeg");
        File::create(&file2).expect("Failed to create file2");

        // Act
        let target_path = tmp_path.to_path_buf();
        let result = get_images_from_target_directory(&target_path, None);

        // Assert
        assert!(result.is_ok(), "Scan should succeed");

        let found_images = result.unwrap();
        assert_eq!(found_images.len(), 2, "Should find exactly 2 images");
        assert!(found_images.contains(&file1), "Should discover file1.png");
        assert!(found_images.contains(&file2), "Should discover file2.png");
    }
}
