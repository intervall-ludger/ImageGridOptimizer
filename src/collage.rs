use image::{DynamicImage, Rgba, GenericImage, GenericImageView};
use rect_packer::Rect;
use std::collections::HashMap;

pub fn create_collage(
    images: &HashMap<u32, DynamicImage>,
    packed_locations: &[(u32, Rect)],
    max_width: u32,
    max_height: u32,
) -> DynamicImage {
    println!("Creating collage...");
    println!("Collage dimensions: Width = {}, Height = {}", max_width, max_height);

    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0;
    let mut max_y = 0;

    for (id, rect) in packed_locations {
        let x_end = (rect.x + rect.width) as u32;
        let y_end = (rect.y + rect.height) as u32;

        if (rect.x as u32) < min_x { min_x = rect.x as u32; }
        if (rect.y as u32) < min_y { min_y = rect.y as u32; }
        if x_end > max_x { max_x = x_end; }
        if y_end > max_y { max_y = y_end; }

        println!(
            "Image ID: {}, Position: ({}, {}), Size: {}x{}",
            id, rect.x, rect.y, rect.width, rect.height
        );
    }

    let bounding_width = max_x - min_x;
    let bounding_height = max_y - min_y;

    let offset_x = (max_width.saturating_sub(bounding_width)) / 2;
    let offset_y = (max_height.saturating_sub(bounding_height)) / 2;

    let mut collage = DynamicImage::new_rgba8(max_width, max_height);

    // Fill background with white
    for y in 0..max_height {
        for x in 0..max_width {
            collage.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // Place images with offset
    for (id, rect) in packed_locations {
        if let Some(img) = images.get(id) {
            let target_x = offset_x + (rect.x as u32 - min_x);
            let target_y = offset_y + (rect.y as u32 - min_y);
            collage.copy_from(img, target_x, target_y).unwrap();
        }
    }

    collage
}
