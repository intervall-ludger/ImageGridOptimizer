use image::imageops;
use image::imageops::resize;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};

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

/// Scales an image by a specific factor.
///
/// # Parameters
/// - `img`: The image to be scaled. Note that we take ownership here.
/// - `scale_factor`: The factor by which the image will be scaled.
///
/// # Returns
/// A new `DynamicImage` representing the scaled image.
fn scale_image(mut img: DynamicImage, scale_factor: f64) -> DynamicImage {
    let (width, height) = img.dimensions();
    let new_width = (width as f64 * scale_factor) as u32;
    let new_height = (height as f64 * scale_factor) as u32;

    img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

    for y in 0..img.height() {
        for x in 0..img.width() {
            if y == 0 || x == 0 || y == img.height()  || x == img.width() {
                img.put_pixel(x, y, image::Rgba([255u8, 255u8, 255u8, 255u8]));
            }
        }
    }
    return img
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
fn create_random_collage(mut images: Vec<DynamicImage>, min_number: usize, max_number: usize) -> (Vec<DynamicImage>, f64) {
    let mode = "random";
    if mode == "area" {
        images.sort_by(|a, b| {
            let area_a = a.dimensions().0 * a.dimensions().1;
            let area_b = b.dimensions().0 * b.dimensions().1;
            area_b.cmp(&area_a)
        });
    } else if mode == "random" {
        let mut rng = thread_rng();
        images.shuffle(&mut rng);
    } else {
        images.sort_by(|a, b| {
            let width_a = a.width();
            let width_b = b.width();
            width_b.cmp(&width_a)
        });
    }
    let mut rng = rand::thread_rng();
    let number: usize = rng.gen_range(min_number..=max_number) as usize;
    images = images[0..number].to_vec();
    let (collage, free_space) = create_collage(images.clone(), 0.1);
    return (images, free_space)
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
fn create_collage(mut images: Vec<DynamicImage>, scaling: f64) -> (DynamicImage, f64) {
    let DEBUG = false;
    let first_image = images.remove(0);
    let mut collage = scale_image(first_image.clone(), scaling);
    let mut white_collage = create_white_dynamic_image(&collage);


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
        (collage, white_collage) = place_image(collage, white_collage, &scale_image(img.clone(), scaling));
        pb.inc(step_size.try_into().unwrap());
        if DEBUG {
            collage.save(format!("collage_step_{}.png", count)).unwrap();
            white_collage.save(format!("white_collage_step_{}.png", count)).unwrap();
            println!("{}", count);
            count += 1;
        }
    }

    pb.finish_with_message("All images processed!");

    collage = fill_background(collage, white_collage.clone());
    (collage, count_free_spaces(&white_collage))
}

fn create_white_dynamic_image(image: &DynamicImage) -> DynamicImage {
    // Create an ImageBuffer with the same dimensions as the collage, filled with white pixels
    let white_image_buffer = ImageBuffer::from_pixel(
        image.width(),
        image.height(),
        Rgba([255u8, 255u8, 255u8, 255u8])
    );

    // Convert the ImageBuffer to a DynamicImage
    DynamicImage::ImageRgba8(white_image_buffer)
}

/// Calculates the proportion of free (empty) spaces within a collage.
///
/// This function scans a given collage image, identifying free spaces based on the color of the pixels. It defines a free space as a pixel with a color value of [0, 0, 0, 255] (solid black). The function then calculates the ratio of these free spaces to the total number of pixels in the collage.
///
/// # Parameters
///
/// - `white_collage`: A reference to a `DynamicImage` representing the collage. The name `white_collage` might be misleading because the function actually looks for black pixels ([0, 0, 0, 255]) to count as free spaces. Consider renaming this parameter to better reflect its purpose or the content it's expected to hold.
///
/// # Returns
///
/// A `f64` representing the proportion of free spaces in the collage. This is calculated as the number of free spaces divided by the total number of pixels in the collage.
///
fn count_free_spaces(white_collage: &DynamicImage) -> f64 {
    let (width, height) = white_collage.dimensions();
    let mut free_spaces = 0.0;

    for y in 0..height {
        for x in 0..width {
            let pixel = white_collage.get_pixel(x, y);
            if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 && pixel[3] == 255 {
                free_spaces += 1.0;
            }
        }
    }

    free_spaces / (height * width) as f64
}

fn fill_background(mut collage: DynamicImage, white_collage: DynamicImage) -> DynamicImage {
    for y in 0..collage.height() {
        for x in 0..collage.width() {
            let pixel = white_collage.get_pixel(x, y);
            if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 && pixel[3] == 255 {
                collage.put_pixel(x, y, image::Rgba([255u8, 255u8, 255u8, 255u8]));
            }
        }
    }
    collage
}


/// Places a new image onto a collage.
///
/// # Parameters
///
/// - `collage`: The existing collage.
/// - `white_collage`: A collage where each image is white (counting free space).
/// - `new_image`: The new image to be placed on the collage.
///
/// # Returns
///
/// A new collage with the new image placed.
fn place_image(mut collage: DynamicImage, mut white_collage: DynamicImage, new_image: &DynamicImage) -> (DynamicImage, DynamicImage) {
    let (width, height) = collage.dimensions();
    let (new_width, new_height) = new_image.dimensions();
    let mut min_width = width;
    let mut min_height = height;
    let aim_rotation = 16.0 / 9.0;
    let mut found = false;
    let mut boundary = false;
    let new_image_white = create_white_dynamic_image(new_image);

    for y in 0..height {
        for x in 0..width {
            boundary = false;
            let pixel = white_collage.get_pixel(x, y);
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
                    let neighbor_pixel = white_collage.get_pixel(nx, ny);
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
                    white_collage.copy_from(&new_image_white, x, y).unwrap();
                    return crop_collage(collage, white_collage)
                }
                let mut tmp_width = x + new_width + 1;
                let mut tmp_height = y + new_height + 1;
                if tmp_width < width {
                    tmp_width = width;
                }
                if tmp_height < height {
                    tmp_height = height;
                }
                let current_rotation = tmp_width as f32 / tmp_height as f32;
                if (current_rotation - aim_rotation).abs() < (min_width as f32 / min_height as f32 - aim_rotation).abs() {
                    min_width = tmp_width;
                    min_height = tmp_height;
                    found = true;
                }
            }
        }
    }
    if found {
        let mut new_collage = DynamicImage::new_rgb8(min_width, min_height);
        let mut new_white_collage = new_collage.clone();
        new_collage.copy_from(&collage, 0, 0).unwrap();
        new_white_collage.copy_from(&white_collage, 0, 0).unwrap();
        place_image(new_collage, new_white_collage, new_image)
    } else {
        if width > height {
            let mut new_collage = DynamicImage::new_rgb8(width, height + new_height - 1);
            let mut new_white_collage = new_collage.clone();
            new_collage.copy_from(&collage, 0, 0).unwrap();
            new_white_collage.copy_from(&white_collage, 0, 0).unwrap();
            return place_image(new_collage, new_white_collage, new_image);
        } else {
            let mut new_collage = DynamicImage::new_rgb8(width + new_width - 1, height);
            let mut new_white_collage = new_collage.clone();
            new_collage.copy_from(&collage, 0, 0).unwrap();
            new_white_collage.copy_from(&white_collage, 0, 0).unwrap();
            return place_image(new_collage, new_white_collage, new_image);
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
/// * `white_collage`: A reference to the `DynamicImage` from which the black border should be removed.
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
fn crop_collage(mut collage: DynamicImage, mut white_collage: DynamicImage) -> (DynamicImage, DynamicImage) {
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
    collage = image::DynamicImage::ImageRgba8(imageops::crop_imm(&collage, min_x, min_y, crop_width, crop_height).to_image());
    white_collage = image::DynamicImage::ImageRgba8(imageops::crop_imm(&white_collage, min_x, min_y, crop_width, crop_height).to_image());
    return (collage, white_collage)
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

/// Processes images from a directory and creates a collage, with customizable parameters for experimentation.
///
/// # Parameters
///
/// - `dir`: The directory containing the images.
/// - `filter`: An optional filter for image extensions or filenames.
/// - `standard_width`: An optional standard width to scale the images to.
/// - `num_trials`: The number of random collages to generate for finding the best one.
/// - `min_images`: The minimum number of images per collage.
/// - `max_images`: The maximum number of images per collage.
///
/// # Returns
///
/// A single image representing the best collage found.
/// Process images to find the best collage based on minimal free space criteria.
pub fn process_images(
    dir: &str,
    filter: Option<String>,
    standard_width: Option<u32>,
    num_trials: usize,
    min_images: usize,
    max_images: usize,
) -> DynamicImage {
    // Load images based on directory, filter, and standard width.
    let images_vec = load_images(dir, filter, standard_width);

    let mut min_space = 1.0;
    let mut count = 1;
    let mut best_collage_images = Vec::new();
    // Generate trials to create random collages based on user-specified numbers.
    while min_space > 0.01 {
        let trials: Vec<usize> = (0..num_trials).collect();
        let mut all_results: Vec<_> = trials.into_par_iter().map(|_| {
            let collage_result = create_random_collage(images_vec.clone(), min_images, max_images);
            collage_result
        })
            .collect();

        // Sort the collages based on minimum free space and take the top N collages.
        all_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_collages = all_results.iter().take(30).cloned().collect::<Vec<_>>();

        // Update the collection of best collage images
        best_collage_images = top_collages;

        // Update min_space to the smallest free space in the top collages
        min_space = best_collage_images
            .first()
            .map(|(_collage, space)| *space)
            .unwrap_or(f64::MAX);
        println!("Min Space: {} - count {}", min_space, count);
        count += 1;
    }

    // Parallel processing of the best collages to find the very best one.
    let min_free_space = Arc::new(Mutex::new(f64::MAX));
    let best_of_best = Arc::new(Mutex::new(None));

    best_collage_images.par_iter().for_each(|best_collage_image| {
        println!("{}", best_collage_image.1); // Print free space for debugging.
        let (collage, free_space) = create_collage(best_collage_image.0.clone(), 1.0);

        let mut min_free_space_guard = min_free_space.lock().unwrap();
        let mut best_of_best_guard = best_of_best.lock().unwrap();

        // Update the best collage if the current one has less free space.
        if free_space < *min_free_space_guard {
            *min_free_space_guard = free_space;
            *best_of_best_guard = Some(collage);
        }
    });

    // Debugging: Print the minimum free space found.
    println!("{:?}", *min_free_space);
    // Handle the case where no best collage is found.
    let final_best_collage = best_of_best.lock().unwrap();
    match &*final_best_collage {
        Some(collage) => collage.clone(),
        None => {
            // Return a default image or handle the error as required.
            DynamicImage::new_rgb8(1, 1) // Placeholder for error handling.
        },
    }
}
