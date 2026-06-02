use std::collections::HashMap;

// Binary slicing tree: every leaf is one image, every cut splits the region.
#[derive(Clone)]
pub enum Slot {
    Leaf { id: u32, rotated: bool },
    Cut { vertical: bool, left: Box<Slot>, right: Box<Slot> },
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub rotated: bool,
}

// Aspect ratio of a subtree, combined bottom-up so the layout preserves every
// image's own ratio: side-by-side adds ratios, stacked adds inverse ratios.
// A rotated leaf contributes the inverse of its native aspect ratio.
pub fn aspect(node: &Slot, aspects: &HashMap<u32, f64>) -> f64 {
    match node {
        Slot::Leaf { id, rotated } => {
            let a = aspects[id];
            if *rotated { 1.0 / a } else { a }
        }
        Slot::Cut { vertical, left, right } => {
            let (al, ar) = (aspect(left, aspects), aspect(right, aspects));
            if *vertical {
                al + ar
            } else {
                1.0 / (1.0 / al + 1.0 / ar)
            }
        }
    }
}

// Hand each subtree an exact pixel box; integer split points are derived as
// differences so neighbours share an edge with no gap and no seam.
pub fn assign(node: &Slot, x: u32, y: u32, w: u32, h: u32, aspects: &HashMap<u32, f64>, out: &mut Vec<Cell>) {
    match node {
        Slot::Leaf { id, rotated } => out.push(Cell { id: *id, x, y, w, h, rotated: *rotated }),
        Slot::Cut { vertical, left, right } => {
            let (al, ar) = (aspect(left, aspects), aspect(right, aspects));
            if *vertical {
                let wl = ((w as f64) * al / (al + ar)).round() as u32;
                assign(left, x, y, wl, h, aspects, out);
                assign(right, x + wl, y, w - wl, h, aspects, out);
            } else {
                // Heights are proportional to the inverse aspect ratio.
                let ht = ((h as f64) * ar / (al + ar)).round() as u32;
                assign(left, x, y, w, ht, aspects, out);
                assign(right, x, y + ht, w, h - ht, aspects, out);
            }
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
