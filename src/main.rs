use image::{DynamicImage, ImageBuffer, Rgba};
use image_hasher::{HashAlg, HasherConfig, ImageHash};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::{env, fs};
use indicatif::{ProgressBar, ProgressStyle};

fn main() {
    println!("Finding files in current directory...");

    let curr_path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => panic!("Failed to read current directory: {}", e),
    };
    let files = get_files_from_path(curr_path);

    println!("Found {} valid image files", files.len());
    println!("Computing hashes...");
    let image_hashes = read_and_hash_files(&files);
    println!("Done!");
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

struct HashedImageEntry {
    path: PathBuf,
    hash: ImageHash
}

fn read_and_hash_files(file_paths: &Vec<PathBuf>) -> Result<Vec<HashedImageEntry>, String> {
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .hash_size(8, 8)
        .to_hasher();

    let mut hashed_image_entries: Vec<HashedImageEntry> = Vec::new();
    let total_files = file_paths.len() as u64;

    let pb = ProgressBar::new(total_files);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) | ETA: {eta}"
        )
            .unwrap()
            .progress_chars("#>-")
    );

    for file_path in file_paths {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            pb.set_message(file_name.to_string());
        }

        let img = load_dynamic_image(file_path)?;
        let img_hash = hasher.hash_image(&img);

        let hashed_image_entry = HashedImageEntry {
            path: file_path.to_path_buf(),
            hash: img_hash
        };

        hashed_image_entries.push(hashed_image_entry);
        pb.inc(1);
    }
    return Ok(hashed_image_entries);
}

fn load_dynamic_image(path: &PathBuf) -> Result<DynamicImage, String> {
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if extension == "heic" || extension == "heif" {
        let bytes = fs::read(path)
            .map_err(|e| format!("failed to read image: {}", e))?;

        let decoded = heic::DecoderConfig::new()
            .decode(&bytes, heic::PixelLayout::Rgba8)
            .map_err(|e| format!("HEIC/HEIF Image Decoding Error: {}", e))?;

        let buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
                decoded.width,
                decoded.height,
                decoded.data,
            ).ok_or("Failed to construct image buffer from HEIC data")?;

        return Ok(DynamicImage::ImageRgba8(buffer));
    }

    image::open(path).map_err(|e| format!("Failed to open image: {}", e))
}
