use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use image::DynamicImage;

use crate::packing::{pack_images, DESIRED_ASPECT_RATIO};

#[derive(Clone)]
pub struct Individual {
    pub image_ids: Vec<u32>,
    pub fitness: f64,
    pub packed_layout: Option<(Vec<(u32, rect_packer::Rect)>, u32, u32)>,
}

pub fn create_random_individual(
    all_images: &[(u32, DynamicImage)],
    min_images: usize,
    max_images: usize,
    rng: &mut impl Rng,
) -> Individual {
    let num_images = (rng.gen_range(min_images..=max_images)).min(all_images.len());
    let mut shuffled = all_images.iter().map(|(id, _)| *id).collect::<Vec<u32>>();
    shuffled.shuffle(rng);
    shuffled.truncate(num_images);

    Individual {
        image_ids: shuffled,
        fitness: 0.0,
        packed_layout: None,
    }
}

pub fn enforce_image_limits(
    image_ids: &mut Vec<u32>,
    all_images: &[(u32, DynamicImage)],
    min_images: usize,
    max_images: usize,
    rng: &mut impl Rng,
) {
    // Ensure at least min_images
    while image_ids.len() < min_images {
        let mut available: Vec<u32> = all_images.iter().map(|(id, _)| *id).collect();
        available.retain(|x| !image_ids.contains(x));
        if available.is_empty() {
            break;
        }
        if let Some(&new_id) = available.choose(rng) {
            image_ids.push(new_id);
        }
    }

    // Ensure no more than max_images
    while image_ids.len() > max_images {
        let remove_idx = rng.gen_range(0..image_ids.len());
        image_ids.remove(remove_idx);
    }
}

pub fn evaluate_individual(
    indiv: &mut Individual,
    all_images_map: &HashMap<u32, DynamicImage>,
) {
    let (packed_locations, w, h) = pack_images(&indiv.image_ids, all_images_map);
    if packed_locations.is_empty() || w == 0 || h == 0 {
        indiv.fitness = 0.0;
        indiv.packed_layout = None;
        return;
    }
    let collage_area = (w as u64) * (h as u64);
    let total_packed_area: u64 = packed_locations
        .iter()
        .map(|(_, rect)| rect.width as u64 * rect.height as u64)
        .sum();
    let free_area = collage_area.saturating_sub(total_packed_area);
    let free_area_percentage = (free_area as f64 / collage_area as f64) * 100.0;
    let aspect_ratio = if h == 0 { 9999.9 } else { w as f64 / h as f64 };
    let aspect_ratio_diff = (aspect_ratio - DESIRED_ASPECT_RATIO).abs();

    let image_count_factor = indiv.image_ids.len() as f64;
    // Fitness function considers number of images, free area, and aspect ratio deviation
    let fitness = image_count_factor / (1.0 + free_area_percentage + aspect_ratio_diff * 10.0);

    indiv.fitness = fitness;
    indiv.packed_layout = Some((packed_locations, w, h));
}

pub fn crossover(
    parent1: &Individual,
    parent2: &Individual,
    all_images: &[(u32, DynamicImage)],
    min_images: usize,
    max_images: usize,
    rng: &mut impl Rng
) -> Individual {
    let p1_len = parent1.image_ids.len();
    let p2_len = parent2.image_ids.len();

    if p1_len == 0 && p2_len == 0 {
        return Individual {
            image_ids: vec![],
            fitness: 0.0,
            packed_layout: None,
        };
    }

    let cutoff_p1 = rng.gen_range(0..=p1_len);
    let cutoff_p2 = rng.gen_range(0..=p2_len);

    let mut child_ids = parent1.image_ids[..cutoff_p1].to_vec();
    child_ids.extend_from_slice(&parent2.image_ids[cutoff_p2..]);

    child_ids.sort();
    child_ids.dedup();

    enforce_image_limits(&mut child_ids, all_images, min_images, max_images, rng);

    Individual {
        image_ids: child_ids,
        fitness: 0.0,
        packed_layout: None,
    }
}

pub fn mutate(
    indiv: &mut Individual,
    all_images: &[(u32, DynamicImage)],
    min_images: usize,
    max_images: usize,
    rng: &mut impl Rng
) {
    if indiv.image_ids.is_empty() {
        return;
    }

    let roll = rng.gen::<f64>();

    if roll < 0.33 && indiv.image_ids.len() < max_images {
        // Add a new image
        let mut available: Vec<u32> = all_images.iter().map(|(id, _)| *id).collect();
        available.retain(|x| !indiv.image_ids.contains(x));
        if let Some(&new_id) = available.choose(rng) {
            indiv.image_ids.push(new_id);
        }
    } else if roll < 0.66 && indiv.image_ids.len() > min_images {
        // Remove an image
        let remove_idx = rng.gen_range(0..indiv.image_ids.len());
        indiv.image_ids.remove(remove_idx);
    } else {
        // Replace an image
        if !all_images.is_empty() {
            let idx = rng.gen_range(0..indiv.image_ids.len());
            let mut available: Vec<u32> = all_images.iter().map(|(id, _)| *id).collect();
            available.retain(|x| !indiv.image_ids.contains(x));
            if let Some(&new_id) = available.choose(rng) {
                indiv.image_ids[idx] = new_id;
            }
        }
    }

    enforce_image_limits(&mut indiv.image_ids, all_images, min_images, max_images, rng);
}
