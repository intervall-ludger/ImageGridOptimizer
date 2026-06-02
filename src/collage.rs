use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use image::imageops::FilterType;
use std::collections::HashMap;

use crate::packing::Cell;

pub fn create_collage(
    images: &HashMap<u32, DynamicImage>,
    cells: &[Cell],
    content_w: u32,
    content_h: u32,
    flex: f64,
    gutter: u32,
    margin: u32,
) -> DynamicImage {
    let canvas_w = content_w + 2 * margin;
    let canvas_h = content_h + 2 * margin;
    println!("Collage: {}x{} ({} images, flex={})", canvas_w, canvas_h, cells.len(), flex);

    let mut collage = DynamicImage::new_rgba8(canvas_w, canvas_h);
    for y in 0..canvas_h {
        for x in 0..canvas_w {
            collage.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // flex=0 draws every image at a common native scale (gaps appear), flex=1
    // fills each cell completely (gap-free). The shared scale k is the largest
    // factor that keeps every native image inside its cell.
    let k = cells
        .iter()
        .map(|c| {
            let (w, h) = images[&c.id].dimensions();
            cell_inner(c, gutter) / (w as f64 * h as f64)
        })
        .fold(f64::MAX, f64::min);

    for cell in cells {
        let rotated_img;
        let img: &DynamicImage = if cell.rotated {
            rotated_img = images[&cell.id].rotate90();
            &rotated_img
        } else {
            &images[&cell.id]
        };
        let (src_w, src_h) = img.dimensions();
        let native_fill = (k * src_w as f64 * src_h as f64 / cell_inner(cell, gutter)).sqrt();
        let fill = native_fill + flex * (1.0 - native_fill);

        // Contain-fit into the gutter-reduced cell preserves the aspect ratio
        // exactly; `fill` then shrinks it toward its native size for low flex.
        let avail_w = cell.w.saturating_sub(gutter) as f64;
        let avail_h = cell.h.saturating_sub(gutter) as f64;
        let scale = (avail_w / src_w as f64).min(avail_h / src_h as f64) * fill;
        let draw_w = (src_w as f64 * scale).round().max(1.0) as u32;
        let draw_h = (src_h as f64 * scale).round().max(1.0) as u32;

        let resized = img.resize_exact(draw_w, draw_h, FilterType::Lanczos3);
        let tx = margin + cell.x + (cell.w.saturating_sub(draw_w)) / 2;
        let ty = margin + cell.y + (cell.h.saturating_sub(draw_h)) / 2;
        collage.copy_from(&resized, tx, ty).unwrap();
    }

    collage
}

fn cell_inner(cell: &Cell, gutter: u32) -> f64 {
    let w = cell.w.saturating_sub(gutter) as f64;
    let h = cell.h.saturating_sub(gutter) as f64;
    (w * h).max(1.0)
}
