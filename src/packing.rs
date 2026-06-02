use std::collections::HashMap;

// Binary slicing tree: every leaf is one image, every cut splits the region.
#[derive(Clone)]
pub enum Slot {
    Leaf { id: u32, rotated: bool },
    Cut { vertical: bool, left: Box<Slot>, right: Box<Slot> },
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
pub struct Cell {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub rotated: bool,
}

struct Raw {
    id: u32,
    rotated: bool,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

// Aspect ratio of a subtree, combined bottom-up; a rotated leaf contributes the
// inverse of its native aspect ratio.
pub fn aspect(node: &Slot, aspects: &HashMap<u32, f64>) -> f64 {
    match node {
        Slot::Leaf { id, rotated } => {
            let a = aspects[id];
            if *rotated { 1.0 / a } else { a }
        }
        Slot::Cut { vertical, left, right } => {
            let (al, ar) = (aspect(left, aspects), aspect(right, aspects));
            if *vertical { al + ar } else { 1.0 / (1.0 / al + 1.0 / ar) }
        }
    }
}

// Fraction of the canvas area each leaf gets in the gap-free tessellation
// (flex=1 target), derived analytically from the aspect split rules.
fn leaf_areas(node: &Slot, area: f64, aspects: &HashMap<u32, f64>, out: &mut Vec<f64>) {
    match node {
        Slot::Leaf { .. } => out.push(area),
        Slot::Cut { vertical, left, right } => {
            let (al, ar) = (aspect(left, aspects), aspect(right, aspects));
            let (fl, fr) = if *vertical {
                (al / (al + ar), ar / (al + ar))
            } else {
                (ar / (al + ar), al / (al + ar))
            };
            leaf_areas(left, area * fl, aspects, out);
            leaf_areas(right, area * fr, aspects, out);
        }
    }
}

pub fn collect_ids(node: &Slot, out: &mut Vec<u32>) {
    match node {
        Slot::Leaf { id, .. } => out.push(*id),
        Slot::Cut { left, right, .. } => {
            collect_ids(left, out);
            collect_ids(right, out);
        }
    }
}

pub fn leaf_count(node: &Slot) -> usize {
    match node {
        Slot::Leaf { .. } => 1,
        Slot::Cut { left, right, .. } => leaf_count(left) + leaf_count(right),
    }
}

// Pack the tree with per-leaf scaling controlled by flex: each image is scaled
// between its original size (flex=0, tightest packing of fixed sizes) and its
// gap-free tessellation size (flex=1). Returns the placed rectangles plus the
// bounding box, all in arbitrary units (the caller scales to the final width).
fn pack(node: &Slot, aspects: &HashMap<u32, f64>, dims: &HashMap<u32, (f64, f64)>, flex: f64) -> (Vec<Raw>, f64, f64) {
    let mut ids = Vec::new();
    collect_ids(node, &mut ids);
    let mut fracs = Vec::new();
    leaf_areas(node, 1.0, aspects, &mut fracs);
    let total_native: f64 = ids.iter().map(|id| dims[id].0 * dims[id].1).sum();

    let mut scaled: HashMap<u32, (f64, f64)> = HashMap::with_capacity(ids.len());
    for (id, &frac) in ids.iter().zip(&fracs) {
        let (w, h) = dims[id];
        let s_tess = (frac * total_native / (w * h)).sqrt();
        // Geometric interpolation between original (flex=0) and gap-free tessellation
        // (flex=1) size; symmetric in log-space so shrinking and growing progress evenly.
        let scale = s_tess.powf(flex);
        scaled.insert(*id, (w * scale, h * scale));
    }

    let mut out = Vec::with_capacity(ids.len());
    let (w, h) = place(node, &scaled, 0.0, 0.0, &mut out);
    (out, w, h)
}

fn measure(node: &Slot, scaled: &HashMap<u32, (f64, f64)>) -> (f64, f64) {
    match node {
        Slot::Leaf { id, rotated } => {
            let (w, h) = scaled[id];
            if *rotated { (h, w) } else { (w, h) }
        }
        Slot::Cut { vertical, left, right } => {
            let (lw, lh) = measure(left, scaled);
            let (rw, rh) = measure(right, scaled);
            if *vertical { (lw + rw, lh.max(rh)) } else { (lw.max(rw), lh + rh) }
        }
    }
}

// Place children flush against each other, centered on the shorter axis; gaps
// only appear where sibling extents differ, which the GA minimises.
fn place(node: &Slot, scaled: &HashMap<u32, (f64, f64)>, x: f64, y: f64, out: &mut Vec<Raw>) -> (f64, f64) {
    match node {
        Slot::Leaf { id, rotated } => {
            let (w, h) = measure(node, scaled);
            out.push(Raw { id: *id, rotated: *rotated, x, y, w, h });
            (w, h)
        }
        Slot::Cut { vertical, left, right } => {
            let (lw, lh) = measure(left, scaled);
            let (rw, rh) = measure(right, scaled);
            if *vertical {
                let h = lh.max(rh);
                place(left, scaled, x, y + (h - lh) / 2.0, out);
                place(right, scaled, x + lw, y + (h - rh) / 2.0, out);
                (lw + rw, h)
            } else {
                let w = lw.max(rw);
                place(left, scaled, x + (w - lw) / 2.0, y, out);
                place(right, scaled, x + (w - rw) / 2.0, y + lh, out);
                (w, lh + rh)
            }
        }
    }
}

// Bounding-box aspect ratio, white fraction, and area-uniformity (coefficient of
// variation of cell areas) of the packed layout. White is minimised directly;
// uniformity is a tie-breaker the GA only weights at high flex, where white is
// already ~0 and a clean even mosaic is preferred.
pub fn layout_metrics(node: &Slot, aspects: &HashMap<u32, f64>, dims: &HashMap<u32, (f64, f64)>, flex: f64) -> (f64, f64, f64) {
    let (cells, w, h) = pack(node, aspects, dims, flex);
    let area = w * h;
    if area <= 0.0 || cells.is_empty() {
        return (1.0, 1.0, 0.0);
    }
    let areas: Vec<f64> = cells.iter().map(|c| c.w * c.h).collect();
    let used: f64 = areas.iter().sum();
    let mean = used / cells.len() as f64;
    let variance = areas.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / cells.len() as f64;
    let cv = if mean > 0.0 { variance.sqrt() / mean } else { 0.0 };
    (w / h, 1.0 - used / area, cv)
}

pub fn layout_cells(
    node: &Slot,
    aspects: &HashMap<u32, f64>,
    dims: &HashMap<u32, (f64, f64)>,
    flex: f64,
    content_w: u32,
) -> (Vec<Cell>, u32, u32) {
    let content_w = content_w.max(1);
    let (raw, w, h) = pack(node, aspects, dims, flex);
    let scale = content_w as f64 / w;
    // Derive each cell from rounded edge coordinates so neighbours tile exactly
    // and no boundary cell overshoots the canvas (right/bottom edge <= content_*).
    let cells = raw
        .iter()
        .map(|c| {
            let x0 = (c.x * scale).round() as u32;
            let y0 = (c.y * scale).round() as u32;
            let x1 = ((c.x + c.w) * scale).round() as u32;
            let y1 = ((c.y + c.h) * scale).round() as u32;
            Cell { id: c.id, x: x0, y: y0, w: (x1 - x0).max(1), h: (y1 - y0).max(1), rotated: c.rotated }
        })
        .collect();
    (cells, content_w, (h * scale).round().max(1.0) as u32)
}
