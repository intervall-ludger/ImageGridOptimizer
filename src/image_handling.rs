use std::fs;
use image::imageops::{resize, FilterType};
use image::{DynamicImage, GenericImageView};

pub fn load_images(dir: &str, filter: Option<String>, standard_width: Option<u32>) -> Vec<(u32, DynamicImage)> {
    println!("Loading images from directory: {}", dir);
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading directory {}: {}", dir, e);
            return Vec::new();
        }
    };

    let mut images = Vec::new();
    let mut id_counter = 0;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading an entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        let passes_filter = if let Some(f) = &filter {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                name.contains(f)
            } else {
                false
            }
        } else {
            true
        };

        if path.is_file() && passes_filter {
            println!("Opening image: {}", path.display());
            let img_result = image::open(&path);
            match img_result {
                Ok(img) => {
                    println!("Successfully opened: {}", path.display());
                    let scaled_img = scale_to_standard_width(&img, standard_width);
                    images.push((id_counter, scaled_img));
                    id_counter += 1;
                }
                Err(e) => {
                    eprintln!("Error opening {}: {}", path.display(), e);
                    continue;
                }
            }
        } else {
            println!("Skipping: {}", path.display());
        }
    }

    println!("Total images loaded: {}", images.len());
    images
}

fn scale_to_standard_width(
    img: &DynamicImage,
    standard_width: Option<u32>,
) -> DynamicImage {
    if let Some(width) = standard_width {
        let (current_width, current_height) = img.dimensions();
        let new_height = (width as f64 / current_width as f64 * current_height as f64) as u32;
        let rgba_img = img.to_rgba8();
        let resized = resize(&rgba_img, width, new_height, FilterType::Lanczos3);
        DynamicImage::ImageRgba8(resized)
    } else {
        img.to_rgba8().into()
    }
}
