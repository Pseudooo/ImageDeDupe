use std::fs;
use std::path::PathBuf;
use image::{DynamicImage, ImageBuffer, Rgba};
use image_hasher::{HashAlg, HasherConfig, ImageHash};

pub struct HashedImageEntry {
    pub path: PathBuf,
    pub hash: ImageHash
}

impl HashedImageEntry {
    pub fn create_from_path(path: &PathBuf) -> Result<HashedImageEntry, String> {
        let hasher = HasherConfig::new()
            .hash_alg(HashAlg::Gradient)
            .hash_size(8, 8)
            .to_hasher();

        let image = load_dynamic_image(path)?;
        let image_hash = hasher.hash_image(&image);

        Ok(HashedImageEntry {
            path: path.to_path_buf(),
            hash: image_hash,
        })
    }
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