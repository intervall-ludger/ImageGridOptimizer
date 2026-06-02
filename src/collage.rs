use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use image::imageops::FilterType;
use std::collections::HashMap;

use crate::packing::Cell;

pub fn create_collage(
    images: &HashMap<u32, DynamicImage>,
    cells: &[Cell],
    content_w: u32,
    content_h: u32,
    gutter: u32,
    margin: u32,
) -> DynamicImage {
    let canvas_w = content_w + 2 * margin;
    let canvas_h = content_h + 2 * margin;
    println!("Collage: {}x{} ({} images)", canvas_w, canvas_h, cells.len());

    let mut collage = DynamicImage::new_rgba8(canvas_w, canvas_h);
    for y in 0..canvas_h {
        for x in 0..canvas_w {
            collage.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    for cell in cells {
        let rotated_img;
        let img: &DynamicImage = if cell.rotated {
            rotated_img = images[&cell.id].rotate90();
            &rotated_img
        } else {
            &images[&cell.id]
        };
        let (src_w, src_h) = img.dimensions();

        // Contain-fit into the gutter-reduced cell; the cell already carries the
        // image's aspect ratio, so this neither stretches nor crops.
        let avail_w = cell.w.saturating_sub(gutter) as f64;
        let avail_h = cell.h.saturating_sub(gutter) as f64;
        let scale = (avail_w / src_w as f64).min(avail_h / src_h as f64);
        let draw_w = (src_w as f64 * scale).round().max(1.0) as u32;
        let draw_h = (src_h as f64 * scale).round().max(1.0) as u32;

        let tx = margin + cell.x + (cell.w.saturating_sub(draw_w)) / 2;
        let ty = margin + cell.y + (cell.h.saturating_sub(draw_h)) / 2;
        // Clamp to the canvas so a boundary cell rounded a pixel too far can never
        // push the paste out of bounds (copy_from would otherwise error).
        let draw_w = draw_w.min(canvas_w.saturating_sub(tx));
        let draw_h = draw_h.min(canvas_h.saturating_sub(ty));
        if draw_w == 0 || draw_h == 0 {
            continue;
        }
        let resized = img.resize_exact(draw_w, draw_h, FilterType::Lanczos3);
        collage.copy_from(&resized, tx, ty).unwrap();
    }

    collage
}
