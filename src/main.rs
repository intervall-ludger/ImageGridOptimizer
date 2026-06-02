use std::collections::HashMap;

mod cli;
mod image_handling;
mod collage;

use crate::cli::parse_args;
use crate::image_handling::load_images;
use crate::collage::create_collage;
use collage_core::{solve, Params};
use image::GenericImageView;

fn main() {
    let cfg = parse_args();
    println!("Loading images from {}...", cfg.dir);
    let images_vec = load_images(&cfg.dir, cfg.filter.clone(), cfg.standard_width);
    if images_vec.is_empty() {
        eprintln!("No images loaded.");
        return;
    }

    let dims: HashMap<u32, (f64, f64)> = images_vec
        .iter()
        .map(|(id, img)| {
            let (w, h) = img.dimensions();
            (*id, (w as f64, h as f64))
        })
        .collect();
    let image_map: HashMap<u32, image::DynamicImage> = images_vec.into_iter().collect();

    let params = Params {
        target_aspect: cfg.target_aspect,
        flex: cfg.flex,
        min_images: cfg.min_images,
        max_images: cfg.max_images,
        allow_rotate: cfg.allow_rotate,
        population_size: cfg.population_size,
        generations: cfg.generations,
        mutation_rate: cfg.mutation_rate,
        width: cfg.width,
        forced: Vec::new(),
    };

    println!("Packing {} images...", dims.len());
    let (cells, content_w, content_h) = solve(&dims, &params);

    // Every image is used at most once: ids start unique and mutations only ever
    // pull from unused ids, so the layout can never repeat an image.
    debug_assert_eq!(
        cells.iter().map(|c| c.id).collect::<std::collections::HashSet<_>>().len(),
        cells.len(),
        "layout contains a duplicate image"
    );

    let collage = create_collage(&image_map, &cells, content_w, content_h, cfg.gutter, cfg.margin);
    println!("Saving image as 'output.jpg'...");
    match collage.save("output.jpg") {
        Ok(_) => println!("Image saved successfully."),
        Err(e) => eprintln!("Error saving image: {}", e),
    }
}
