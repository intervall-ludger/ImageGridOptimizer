//! Contains the main logic for loading, processing, and combining images into an optimized collage.
//!
//! This version distributes padding evenly between images without adding padding around the collage edges.

use image::imageops::resize;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use rand::seq::SliceRandom;
use rand::Rng;
use rect_packer::Packer;
use rect_packer::Rect;
use rect_packer::Config;
use std::fs;
use std::collections::HashMap;

// Define the size of the padding between images (in pixels).
const PADDING_SIZE: u32 = 5; // Size of the padding between images

// Define the desired aspect ratio for the collage (1.0 for square).
const DESIRED_ASPECT_RATIO: f64 = 1.0;

/// Loads images from a directory with an optional filter and scales them to an optional standard width.
///
/// # Parameters
///
/// - `dir`: The directory containing the images.
/// - `filter`: An optional filter for file extensions or filenames.
/// - `standard_width`: An optional standard width to scale images to.
///
/// # Returns
///
/// A vector of loaded images along with their IDs.
fn load_images(
    dir: &str,
    filter: Option<String>,
    standard_width: Option<u32>,
) -> Vec<(u32, DynamicImage)> {
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
                eprintln!("Error reading entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        if path.is_file()
            && (filter.is_none()
            || path
            .extension()
            .and_then(|s| s.to_str())
            .map_or(false, |ext| ext.eq_ignore_ascii_case(filter.as_ref().unwrap())))
        {
            println!("Opening image: {}", path.display());
            let img_result = image::open(&path);
            match img_result {
                Ok(img) => {
                    println!("Image successfully opened: {}", path.display());
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
            println!("Skipping file: {}", path.display());
        }
    }

    println!("Total images loaded: {}", images.len());
    images
}

/// Scales an image to a standard width while maintaining aspect ratio.
/// If no standard width is provided, the image remains unchanged.
///
/// # Parameters
///
/// - `img`: The image to scale.
/// - `standard_width`: An optional standard width to scale the image to.
///
/// # Returns
///
/// A new `DynamicImage` scaled to the standard width or the original image if no width is provided.
fn scale_to_standard_width(
    img: &DynamicImage,
    standard_width: Option<u32>,
) -> DynamicImage {
    if let Some(width) = standard_width {
        let (current_width, current_height) = img.dimensions();

        // Calculate the new height while maintaining aspect ratio.
        let new_height = (width as f64 / current_width as f64 * current_height as f64) as u32;

        // Convert to RGBA8 to ensure consistent pixel format.
        let rgba_img = img.to_rgba8();

        // Scale the image.
        println!(
            "Scaling image to width: {} (Original width: {})",
            width, current_width
        );
        let resized = resize(&rgba_img, width, new_height, FilterType::Lanczos3);

        // Wrap the resized image in DynamicImage.
        DynamicImage::ImageRgba8(resized)
    } else {
        // If no scaling is needed, convert to RGBA8 to maintain consistency.
        img.to_rgba8().into()
    }
}

/// Processes images from a directory and creates an optimized collage with configurable parameters.
///
/// # Parameters
///
/// - `dir`: The directory containing the images.
/// - `filter`: An optional filter for file extensions or filenames.
/// - `standard_width`: An optional standard width to scale images to.
/// - `num_trials`: The number of trials to generate collages.
/// - `min_images`: The minimum number of images per collage.
/// - `max_images`: The maximum number of images per collage.
///
/// # Returns
///
/// A single collage from the selected images.
pub fn process_images(
    dir: &str,
    filter: Option<String>,
    standard_width: Option<u32>,
    num_trials: usize,
    min_images: usize,
    max_images: usize,
) -> DynamicImage {
    println!("Processing images...");
    // Load images.
    let images_vec = load_images(dir, filter, standard_width);
    if images_vec.is_empty() {
        eprintln!("No images found to process.");
        return DynamicImage::new_rgba8(1, 1); // Placeholder for error handling.
    }

    let mut rng = rand::thread_rng();

    // Track the best layout
    let mut best_layout: Option<(Vec<(u32, Rect)>, u32, u32)> = None;
    let mut minimal_free_area = u64::MAX;
    let mut best_aspect_ratio_diff = f64::MAX;

    println!("Starting {} trials to find the optimal collage layout...", num_trials);

    for trial in 1..=num_trials {
        // Randomly select the number of images for this trial
        let num_images = rng.gen_range(min_images..=max_images);
        if num_images > images_vec.len() {
            // Skip trial if not enough images
            continue;
        }

        // Shuffle and select images
        let mut selected_images = images_vec.clone();
        selected_images.shuffle(&mut rng);
        selected_images.truncate(num_images);

        // Calculate total area of selected images
        let total_images_area: u64 = selected_images
            .iter()
            .map(|(_, img)| img.width() as u64 * img.height() as u64)
            .sum();

        // Estimate canvas size based on desired aspect ratio
        let estimated_height = ((total_images_area as f64 / DESIRED_ASPECT_RATIO).sqrt()) as u32;
        let estimated_width = (DESIRED_ASPECT_RATIO * (total_images_area as f64 / DESIRED_ASPECT_RATIO).sqrt()) as u32;

        // Pack the images
        let (packed_locations, packed_width, packed_height) = pack_images(&selected_images, estimated_width, estimated_height);

        // Calculate collage area
        let collage_area = packed_width as u64 * packed_height as u64;

        // Calculate free space
        let free_area = collage_area - total_images_area;

        // Calculate aspect ratio difference from desired
        let aspect_ratio = if packed_height == 0 {
            f64::MAX
        } else {
            (packed_width as f64) / (packed_height as f64)
        };
        let aspect_ratio_diff = (aspect_ratio - DESIRED_ASPECT_RATIO).abs();

        // Determine if this layout is better
        // Prioritize minimal free area, then aspect ratio closeness
        if free_area < minimal_free_area
            || (free_area == minimal_free_area && aspect_ratio_diff < best_aspect_ratio_diff)
        {
            minimal_free_area = free_area;
            best_aspect_ratio_diff = aspect_ratio_diff;
            best_layout = Some((packed_locations.clone(), packed_width, packed_height));

            // Optional: Print progress on finding a new best layout
            println!(
                "Trial {}: New best layout found with free area {} and aspect ratio difference {:.4}",
                trial, free_area, aspect_ratio_diff
            );
        }

        // Optional: Progress indicator every 1000 trials
        if trial % 1000 == 0 {
            println!("Completed {} trials...", trial);
        }
    }

    println!("Trials completed.");

    // Use the best layout found
    if let Some((packed_locations, max_width, max_height)) = best_layout {
        println!(
            "Best layout has free area: {} with collage dimensions: {}x{}",
            minimal_free_area, max_width, max_height
        );
        create_collage(images_vec, packed_locations, max_width, max_height)
    } else {
        eprintln!("No suitable layout found.");
        DynamicImage::new_rgba8(1, 1) // Placeholder for error handling.
    }
}

/// Packs the images using the RectPacker algorithm with desired canvas size.
///
/// # Parameters
///
/// - `images`: A vector of tuples containing image IDs and images.
/// - `estimated_width`: The estimated width of the canvas based on desired aspect ratio.
/// - `estimated_height`: The estimated height of the canvas based on desired aspect ratio.
///
/// # Returns
///
/// A tuple containing:
/// - A vector of tuples with image IDs and their packed rectangles
/// - The maximum width of the collage
/// - The maximum height of the collage
fn pack_images(
    images: &Vec<(u32, DynamicImage)>,
    estimated_width: u32,
    estimated_height: u32,
) -> (Vec<(u32, Rect)>, u32, u32) {
    // Initialize maximum width and height
    let mut max_width = 0;
    let mut max_height = 0;

    // Configuration for the packer
    let config = Config {
        width: estimated_width.max(1500) as i32,  // Ensure a minimum width
        height: estimated_height.max(1500) as i32, // Ensure a minimum height
        border_padding: 0, // No padding around the entire collage
        rectangle_padding: PADDING_SIZE as i32, // Padding between images
    };

    let mut packer = Packer::new(config);

    let mut packed_locations = Vec::new();

    for (id, img) in images {
        let (width, height) = img.dimensions();
        let rectangle = packer.pack(width as i32, height as i32, false);
        match rectangle {
            Some(rect) => {
                packed_locations.push((*id, rect));
                // Update maximum dimensions
                if (rect.x + rect.width) as u32 > max_width {
                    max_width = (rect.x + rect.width) as u32;
                }
                if (rect.y + rect.height) as u32 > max_height {
                    max_height = (rect.y + rect.height) as u32;
                }
            }
            None => {
                // If packing fails, skip this image
                continue;
            }
        }
    }

    (packed_locations, max_width, max_height)
}

/// Creates the collage based on the packed positions.
///
/// # Parameters
///
/// - `images`: A vector of tuples containing image IDs and images.
/// - `packed_locations`: A vector of tuples containing image IDs and their packed rectangles.
/// - `max_width`: The maximum width of the collage.
/// - `max_height`: The maximum height of the collage.
///
/// # Returns
///
/// The final collage as a `DynamicImage`.
fn create_collage(
    images: Vec<(u32, DynamicImage)>,
    packed_locations: Vec<(u32, Rect)>,
    max_width: u32,
    max_height: u32,
) -> DynamicImage {
    println!("Creating collage based on packed positions...");

    println!("Collage dimensions: Width = {}, Height = {}", max_width, max_height);

    // Create a new collage image with a white background.
    let mut collage = DynamicImage::new_rgba8(max_width, max_height);

    // Fill the collage with a white background.
    for y in 0..max_height {
        for x in 0..max_width {
            collage.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // Create a HashMap for quick access to images by ID.
    let image_map: HashMap<u32, DynamicImage> = images.into_iter().collect();

    // Place images onto the collage.
    for (id, rect) in packed_locations {
        if let Some(img) = image_map.get(&id) {
            println!(
                "Placing image ID {} at position ({}, {})",
                id, rect.x, rect.y
            );
            collage
                .copy_from(
                    img,
                    rect.x as u32,
                    rect.y as u32,
                )
                .unwrap();
        }
    }

    collage
}

