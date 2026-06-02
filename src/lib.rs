pub mod packing;
pub mod ga;

#[cfg(feature = "wasm")]
pub mod wasm;

use std::collections::HashMap;

use crate::ga::{create_random_individual, evaluate_individual, mutate, Individual};
use crate::packing::{layout_cells, Cell};
use rand::seq::SliceRandom;
use rand::Rng;

pub struct Params {
    pub target_aspect: f64,
    pub flex: f64,
    pub min_images: usize,
    pub max_images: usize,
    pub allow_rotate: bool,
    pub population_size: usize,
    pub generations: usize,
    pub mutation_rate: f64,
    pub width: u32,
}

// Run the genetic algorithm over the image dimensions and return the packed
// layout (cells) plus the content canvas size. Pure computation: no image
// decoding, no file IO — so it runs natively and in WebAssembly alike.
pub fn solve(dims: &HashMap<u32, (f64, f64)>, params: &Params) -> (Vec<Cell>, u32, u32) {
    let aspects: HashMap<u32, f64> = dims.iter().map(|(id, (w, h))| (*id, w / h)).collect();
    let pool: Vec<u32> = dims.keys().copied().collect();

    let max_images = params.max_images.min(pool.len()).max(1);
    let min_images = params.min_images.min(max_images).max(1);

    let mut rng = rand::thread_rng();
    let mut population: Vec<Individual> = (0..params.population_size.max(1))
        .map(|_| create_random_individual(&pool, min_images, max_images, &mut rng))
        .collect();
    eval_all(&mut population, &aspects, dims, params.target_aspect, params.flex);

    for _ in 1..=params.generations {
        population.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
        let half = (population.len() / 2).max(1);
        let elites = population[..half].to_vec();

        let mut next = elites.clone();
        while next.len() < population.len() {
            let mut child = elites.choose(&mut rng).unwrap().clone();
            mutate(&mut child, &pool, min_images, max_images, params.allow_rotate, &mut rng);
            while rng.gen::<f64>() < params.mutation_rate {
                mutate(&mut child, &pool, min_images, max_images, params.allow_rotate, &mut rng);
            }
            next.push(child);
        }
        eval_all(&mut next, &aspects, dims, params.target_aspect, params.flex);
        population = next;
    }

    population.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
    layout_cells(&population[0].tree, &aspects, dims, params.flex, params.width)
}

#[cfg(feature = "parallel")]
fn eval_all(pop: &mut [Individual], aspects: &HashMap<u32, f64>, dims: &HashMap<u32, (f64, f64)>, target_aspect: f64, flex: f64) {
    use rayon::prelude::*;
    pop.par_iter_mut().for_each(|ind| evaluate_individual(ind, aspects, dims, target_aspect, flex));
}

#[cfg(not(feature = "parallel"))]
fn eval_all(pop: &mut [Individual], aspects: &HashMap<u32, f64>, dims: &HashMap<u32, (f64, f64)>, target_aspect: f64, flex: f64) {
    pop.iter_mut().for_each(|ind| evaluate_individual(ind, aspects, dims, target_aspect, flex));
}
