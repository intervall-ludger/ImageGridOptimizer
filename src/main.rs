use std::collections::HashMap;

mod cli;
mod image_handling;
mod ga;
mod packing;
mod collage;

use crate::cli::parse_args;
use crate::image_handling::load_images;
use crate::ga::{create_random_individual, evaluate_individual, mutate, Individual};
use crate::packing::layout_cells;
use crate::collage::create_collage;
use image::GenericImageView;
use rand::seq::SliceRandom;
use rand::Rng;
use rayon::prelude::*;

fn main() {
    let cfg = parse_args();
    println!("Loading images from {}...", cfg.dir);
    let images_vec = load_images(&cfg.dir, cfg.filter.clone(), cfg.standard_width);
    if images_vec.is_empty() {
        eprintln!("No images loaded.");
        return;
    }

    let aspects: HashMap<u32, f64> = images_vec
        .iter()
        .map(|(id, img)| {
            let (w, h) = img.dimensions();
            (*id, w as f64 / h as f64)
        })
        .collect();
    let dims: HashMap<u32, (f64, f64)> = images_vec
        .iter()
        .map(|(id, img)| {
            let (w, h) = img.dimensions();
            (*id, (w as f64, h as f64))
        })
        .collect();
    let image_map: HashMap<u32, image::DynamicImage> = images_vec.into_iter().collect();
    let pool: Vec<u32> = image_map.keys().copied().collect();

    let max_images = cfg.max_images.min(pool.len());
    let min_images = cfg.min_images.min(max_images).max(1);

    let mut rng = rand::thread_rng();
    let mut population: Vec<Individual> = (0..cfg.population_size)
        .map(|_| create_random_individual(&pool, min_images, max_images, &mut rng))
        .collect();
    population.par_iter_mut().for_each(|ind| evaluate_individual(ind, &aspects, &dims, cfg.target_aspect, cfg.flex));

    for gen in 1..=cfg.generations {
        population.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
        if gen % 50 == 0 || gen == 1 {
            println!("Generation {}: Best fitness = {:.5}", gen, population[0].fitness);
        }

        let half = (cfg.population_size / 2).max(1);
        let elites = population[..half].to_vec();

        let mut next = elites.clone();
        while next.len() < cfg.population_size {
            let mut child = elites.choose(&mut rng).unwrap().clone();
            mutate(&mut child, &pool, min_images, max_images, cfg.allow_rotate, &mut rng);
            while rng.gen::<f64>() < cfg.mutation_rate {
                mutate(&mut child, &pool, min_images, max_images, cfg.allow_rotate, &mut rng);
            }
            next.push(child);
        }

        next.par_iter_mut().for_each(|ind| evaluate_individual(ind, &aspects, &dims, cfg.target_aspect, cfg.flex));
        population = next;
    }

    population.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
    let best = &population[0];
    println!("Best solution fitness: {:.5}", best.fitness);

    let (cells, content_w, content_h) = layout_cells(&best.tree, &aspects, &dims, cfg.flex, cfg.width);

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
