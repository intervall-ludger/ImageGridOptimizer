use image::imageops;
use image::imageops::resize;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

/// Adds a white border around the given image.
///
/// # Parameters
///
/// - `img`: The image to which the border will be added.
/// - `border_size`: The size of the border to be added.
///
/// # Returns
///
/// A new image with the added border.
fn add_white_border(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, border_size: u32) -> DynamicImage {
    let (width, height) = img.dimensions();
    let new_width = width + 2 * border_size;
    let new_height = height + 2 * border_size;

    let mut new_img = DynamicImage::new_rgba8(new_width, new_height);

    // Fill the entire image with white color
    for y in 0..new_height {
        for x in 0..new_width {
            new_img.put_pixel(x, y, image::Rgba([255u8, 255u8, 255u8, 255u8]));
        }
    }

    // Copy the original image to the center of the new image
    imageops::overlay(&mut new_img, img, border_size as i64, border_size as i64);

    new_img
}

/// Loads images from a directory with an optional filter and scales them to an optional standard width.
///
/// # Parameters
///
/// - `dir`: The directory containing the images.
/// - `filter`: An optional filter for image extensions or filenames.
/// - `standard_width`: An optional standard width to scale the images to.
///
/// # Returns
///
/// A vector of loaded images.
fn load_images(
    dir: &str,
    filter: Option<String>,
    standard_width: Option<u32>,
) -> Vec<DynamicImage> {
    const BORDER_SIZE: u32 = 5; // Size of the white border

    fs::read_dir(dir)
        .expect("Failed to read directory")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_file()
                && (filter.is_none()
                || path
                .extension()
                .and_then(|s| s.to_str())
                .map_or(false, |ext| ext == filter.as_ref().unwrap()))
            {
                let img_result = image::open(&path);
                match img_result {
                    Ok(img) => {
                        let scaled_img = scale_to_standard_width(&img, standard_width);
                        Some(add_white_border(&scaled_img, BORDER_SIZE))
                    },
                    Err(e) => {
                        eprintln!("Failed to open {}: {}", path.display(), e);
                        None
                    }
                }
            } else {
                None
            }
        })
        .collect()
}


/// Creates a collage from a vector of images.
///
/// # Parameters
///
/// - `images`: A vector of images to be used in the collage.
///
/// # Returns
///
/// A single image representing the collage.
fn create_collage(mut images: Vec<DynamicImage>) -> DynamicImage {
    let DEBUG = false;
    let mode = "area";
    if mode == "area" {
        images.sort_by(|a, b| {
            let area_a = a.dimensions().0 * a.dimensions().1;
            let area_b = b.dimensions().0 * b.dimensions().1;
            area_b.cmp(&area_a)
        });
    } else {
        images.sort_by(|a, b| {
            let width_a = a.width();
            let width_b = b.width();
            width_b.cmp(&width_a)
        });
    }

    let first_image = images.remove(0);
    let mut collage = first_image;

    let step_size = 100 / images.len();
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% ({eta})")
            .expect("REASON")
            .progress_chars("#>-"),
    );

    let mut count = 1;
    for img in &images {
        collage = place_image(collage, img);
        pb.inc(step_size.try_into().unwrap());
        if DEBUG {
            collage.save(format!("collage_step_{}.png", count)).unwrap();
            println!("{}", count);
            count += 1;
        }
    }

    pb.finish_with_message("All images processed!");

    collage
}

/// Places a new image onto a collage.
///
/// # Parameters
///
/// - `collage`: The existing collage.
/// - `new_image`: The new image to be placed on the collage.
///
/// # Returns
///
/// A new collage with the new image placed.
fn place_image(mut collage: DynamicImage, new_image: &DynamicImage) -> DynamicImage {
    let (width, height) = collage.dimensions();
    let (new_width, new_height) = new_image.dimensions();
    let mut min_width = width;
    let mut min_height = height;
    let mut min_scope = new_width * new_height;
    let mut found = false;
    let mut boundary = false;

    for y in 0..height {
        for x in 0..width {
            boundary = false;
            let pixel = collage.get_pixel(x, y);
            if pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 {
                continue;
            }
            // Check the neighbors
            let neighbors = [
                (x.saturating_sub(1), y), // Left
                (x + 1, y),               // Right
                (x, y.saturating_sub(1)), // Above
                (x, y + 1),               // Below
            ];

            for &(nx, ny) in &neighbors {
                if nx < width && ny < height {
                    let neighbor_pixel = collage.get_pixel(nx, ny);
                    if neighbor_pixel[0] == 255
                        && neighbor_pixel[1] == 255
                        && neighbor_pixel[2] == 255
                    {
                        boundary = true;
                    }
                }
            }
            if !boundary {
                continue;
            }
            if is_empty_space(&collage, x, y, new_width, new_height) {
                if x + new_width <= width && y + new_height <= height {
                    collage.copy_from(new_image, x, y).unwrap();
                    collage = crop_collage(collage);
                    return collage;
                }
                let mut tmp_width = x + new_width + 1;
                let mut tmp_height = y + new_height + 1;
                if tmp_width < width {
                    tmp_width = width;
                }
                if tmp_height < height {
                    tmp_height = height;
                }
                let scope_delta = (tmp_height * tmp_width) - (width * height);
                if scope_delta < min_scope {
                    min_width = tmp_width;
                    min_height = tmp_height;
                    found = true;
                    min_scope = scope_delta;
                }
            }
        }
    }
    if found {
        let mut new_collage = DynamicImage::new_rgb8(min_width, min_height);
        new_collage.copy_from(&collage, 0, 0).unwrap();
        place_image(new_collage, new_image)
    } else {
        if width > height {
            let mut new_collage = DynamicImage::new_rgb8(width, height + new_height - 1);
            new_collage.copy_from(&collage, 0, 0).unwrap();
            return place_image(new_collage, new_image);
        } else {
            let mut new_collage = DynamicImage::new_rgb8(width + new_width - 1, height);
            new_collage.copy_from(&collage, 0, 0).unwrap();
            return place_image(new_collage, new_image);
        }
    }
}

/// `crop_collage` Function
///
/// This function removes the black border from a given collage. It analyzes the image,
/// identifies the non-black area, and returns a new `DynamicImage` that has the black border removed.
///
/// # Parameters
///
/// * `collage`: A reference to the `DynamicImage` from which the black border should be removed.
///
/// # Returns
///
/// Returns a new `DynamicImage` with the black border removed.
///
/// # Example
///
/// ```rust
/// let cropped_collage = crop_collage(&collage);
/// ```
fn crop_collage(collage: DynamicImage) -> DynamicImage {
    let (width, height) = collage.dimensions();

    // Find the bounds of the non-black area
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;

    for y in 0..height {
        for x in 0..width {
            let pixel = collage.get_pixel(x, y);
            if pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 {
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
    }

    // Calculate the dimensions of the cropped area
    let crop_width = (max_x - min_x) + 1;
    let crop_height = (max_y - min_y) + 1;

    // Crop the collage
    image::DynamicImage::ImageRgba8(imageops::crop_imm(&collage, min_x, min_y, crop_width, crop_height).to_image())

}


/// Checks if a space in the collage is empty.
///
/// # Parameters
///
/// - `collage`: The existing collage.
/// - `x`: The x-coordinate of the top-left corner of the space.
/// - `y`: The y-coordinate of the top-left corner of the space.
/// - `width`: The width of the space.
/// - `height`: The height of the space.
///
/// # Returns
///
/// A boolean indicating if the space is empty.
fn is_empty_space(collage: &DynamicImage, x: u32, y: u32, mut width: u32, mut height: u32) -> bool {
    let (collage_width, collage_height) = collage.dimensions();

    if x + width > collage_width {
        width = collage_width - x;
    }
    if y + height > collage_height {
        height = collage_height - y;
    }

    for j in y..(y + height) {
        for i in x..(x + width) {
            let pixel = collage.get_pixel(i, j);
            if pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 {
                return false;
            }
        }
    }
    true
}

/// Scales an image to a standard width while maintaining its aspect ratio.
/// If no standard width is provided, the image remains unchanged.
///
/// # Parameters
///
/// - `img`: The image to be scaled.
/// - `standard_width`: An optional standard width to scale the image to.
///
/// # Returns
///
/// A new image scaled to the standard width or the original image if no standard width is provided.
fn scale_to_standard_width(
    img: &DynamicImage,
    standard_width: Option<u32>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    if let Some(width) = standard_width {
        let (current_width, current_height) = img.dimensions();

        // Calculate the new height while maintaining the aspect ratio.
        let new_height = (width as f64 / current_width as f64 * current_height as f64) as u32;

        // Resize the image.
        resize(img, width, new_height, FilterType::Lanczos3)
    } else {
        img.to_rgba8()
    }
}

/// Processes images from a directory and creates a collage.
///
/// # Parameters
///
/// - `dir`: The directory containing the images.
/// - `filter`: An optional filter for image extensions or filenames.
///
/// # Returns
///
/// A single image representing the collage.
pub fn process_images(
    dir: &str,
    filter: Option<String>,
    standard_width: Option<u32>,
) -> DynamicImage {
    let images_vec = load_images(dir, filter, standard_width);
    create_collage(images_vec)
}
