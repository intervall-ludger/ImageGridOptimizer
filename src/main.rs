use std::collections::HashMap;

mod cli;
mod image_handling;
mod ga;
mod packing;
mod collage;

use crate::cli::parse_args;
use crate::image_handling::load_images;
use crate::ga::{create_random_individual, evaluate_individual, crossover, mutate, enforce_image_limits, Individual};
use crate::collage::create_collage;
use crate::packing::DESIRED_ASPECT_RATIO;
use rand::seq::SliceRandom;
use rand::Rng;
use rayon::prelude::*;

fn main() {
    let (dir, filter, standard_width, population_size, generations, min_images, max_images, mutation_rate, crossover_rate) = parse_args();
    println!("Parameters:");
    println!("Directory: {}", dir);
    println!("Filter: {:?}", filter);
    println!("Standard width: {:?}", standard_width);
    println!("Population size: {}", population_size);
    println!("Generations: {}", generations);
    println!("min_images: {}", min_images);
    println!("max_images: {}", max_images);
    println!("Mutation rate: {}", mutation_rate);
    println!("Crossover rate: {}", crossover_rate);
    println!("Desired aspect ratio: {}", DESIRED_ASPECT_RATIO);

    println!("Loading images...");
    let images_vec = load_images(&dir, filter, standard_width);
    if images_vec.is_empty() {
        eprintln!("No images loaded.");
        return;
    }

    let image_map: HashMap<u32, image::DynamicImage> = images_vec.into_iter().collect();
    let mut rng = rand::thread_rng();

    let all_images = image_map.iter().map(|(id,i)|(id.clone(),i.clone())).collect::<Vec<_>>();
    let mut population: Vec<Individual> = (0..population_size)
        .map(|_| create_random_individual(&all_images, min_images, max_images, &mut rng))
        .collect();

    // Evaluate initial population in parallel
    population.par_iter_mut().for_each(|indiv| {
        evaluate_individual(indiv, &image_map);
    });

    // GA main loop
    for gen in 1..=generations {
        population.sort_by(|a,b| b.fitness.partial_cmp(&a.fitness).unwrap());
        println!("Generation {}: Best fitness = {:.5}", gen, population[0].fitness);

        let half = population_size/2;
        let elites = &population[..half];

        let mut new_population = Vec::new();
        // Keep elites
        new_population.extend_from_slice(elites);

        // Create new individuals
        while new_population.len() < population_size {
            let parent1 = elites.choose(&mut rng).unwrap();
            let parent2 = elites.choose(&mut rng).unwrap();

            let mut child = if rng.gen::<f64>() < crossover_rate {
                crossover(parent1, parent2, &all_images, min_images, max_images, &mut rng)
            } else {
                let mut c = parent1.clone();
                enforce_image_limits(&mut c.image_ids, &all_images, min_images, max_images, &mut rng);
                c
            };

            if rng.gen::<f64>() < mutation_rate {
                mutate(&mut child, &all_images, min_images, max_images, &mut rng);
            }

            new_population.push(child);
        }

        // Evaluate the new population in parallel
        new_population.par_iter_mut().for_each(|indiv| {
            evaluate_individual(indiv, &image_map);
        });

        population = new_population;
    }

    // Final solution
    population.sort_by(|a,b| b.fitness.partial_cmp(&a.fitness).unwrap());
    let best = &population[0];
    println!("Best solution fitness: {:.5}", best.fitness);

    if let Some((packed_locations, w, h)) = &best.packed_layout {
        let collage = create_collage(&image_map, packed_locations, *w, *h);
        println!("Saving image as 'output.jpg'...");
        match collage.save("output.jpg") {
            Ok(_) => println!("Image saved successfully."),
            Err(e) => eprintln!("Error saving image: {}", e),
        }
    } else {
        eprintln!("No layout found for the best solution.");
    }
}
