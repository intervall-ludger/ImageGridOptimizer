use std::collections::HashMap;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use crate::{solve, Params};

#[derive(Deserialize)]
struct InImage {
    id: u32,
    w: f64,
    h: f64,
}

#[derive(Deserialize)]
struct InParams {
    target_aspect: f64,
    flex: f64,
    min_images: usize,
    max_images: usize,
    allow_rotate: bool,
    population_size: usize,
    generations: usize,
    mutation_rate: f64,
    width: u32,
    #[serde(default)]
    forced: Vec<u32>,
}

// Browser entry point: takes image dimensions and parameters as JSON, runs the
// same genetic algorithm as the CLI, and returns the packed cells as JSON.
#[wasm_bindgen]
pub fn solve_collage(images_json: &str, params_json: &str) -> Result<String, JsError> {
    let images: Vec<InImage> = serde_json::from_str(images_json).map_err(|e| JsError::new(&e.to_string()))?;
    let p: InParams = serde_json::from_str(params_json).map_err(|e| JsError::new(&e.to_string()))?;

    let dims: HashMap<u32, (f64, f64)> = images.into_iter().map(|i| (i.id, (i.w.max(1.0), i.h.max(1.0)))).collect();
    if dims.is_empty() {
        return Err(JsError::new("no images provided"));
    }

    let params = Params {
        target_aspect: if p.target_aspect > 0.0 { p.target_aspect } else { 1.0 },
        flex: p.flex.clamp(0.0, 1.0),
        min_images: p.min_images,
        max_images: p.max_images,
        allow_rotate: p.allow_rotate,
        population_size: p.population_size,
        generations: p.generations,
        mutation_rate: p.mutation_rate,
        width: p.width,
        forced: p.forced,
    };

    let (cells, width, height) = solve(&dims, &params);
    let out = serde_json::json!({ "cells": cells, "width": width, "height": height });
    Ok(out.to_string())
}
