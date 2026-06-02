use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

use crate::packing::{aspect, leaf_count, collect_ids, Slot};

const ASPECT_WEIGHT: f64 = 2.0;
const BALANCE_WEIGHT: f64 = 3.0;

#[derive(Clone)]
pub struct Individual {
    pub tree: Slot,
    pub fitness: f64,
}

pub fn create_random_individual(
    pool: &[u32],
    min_images: usize,
    max_images: usize,
    rng: &mut impl Rng,
) -> Individual {
    let n = rng.gen_range(min_images..=max_images).clamp(1, pool.len());
    let mut ids: Vec<u32> = pool.to_vec();
    ids.shuffle(rng);
    ids.truncate(n);
    // Half the population starts from balanced trees (which tile uniform images
    // into a clean grid); the rest start random for mosaic diversity.
    let tree = if rng.gen::<bool>() {
        build_balanced_tree(&ids, rng.gen::<bool>())
    } else {
        build_random_tree(&ids, rng)
    };
    Individual { tree, fitness: 0.0 }
}

fn build_balanced_tree(ids: &[u32], vertical: bool) -> Slot {
    if ids.len() == 1 {
        return Slot::Leaf { id: ids[0], rotated: false };
    }
    let mid = ids.len() / 2;
    Slot::Cut {
        vertical,
        left: Box::new(build_balanced_tree(&ids[..mid], !vertical)),
        right: Box::new(build_balanced_tree(&ids[mid..], !vertical)),
    }
}

fn build_random_tree(ids: &[u32], rng: &mut impl Rng) -> Slot {
    if ids.len() == 1 {
        return Slot::Leaf { id: ids[0], rotated: false };
    }
    let split = rng.gen_range(1..ids.len());
    Slot::Cut {
        vertical: rng.gen::<bool>(),
        left: Box::new(build_random_tree(&ids[..split], rng)),
        right: Box::new(build_random_tree(&ids[split..], rng)),
    }
}

// Fitness rewards using many images while keeping the overall aspect ratio near
// the target and the image areas balanced (so no single image dwarfs the rest).
pub fn evaluate_individual(indiv: &mut Individual, aspects: &HashMap<u32, f64>, target_aspect: f64) {
    let a_root = aspect(&indiv.tree, aspects);
    let mut areas = Vec::new();
    leaf_areas(&indiv.tree, 1.0, aspects, &mut areas);

    let aspect_penalty = (a_root / target_aspect).ln().abs();
    let (min_a, max_a) = areas.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &a| (lo.min(a), hi.max(a)));
    let balance_penalty = (max_a / min_a).ln();

    let n = areas.len() as f64;
    indiv.fitness = n / (1.0 + aspect_penalty * ASPECT_WEIGHT + balance_penalty * BALANCE_WEIGHT);
}

// Fraction of the canvas area each leaf receives, derived analytically from the
// same split rules used in packing::assign (scale-invariant, no pixel rounding).
fn leaf_areas(node: &Slot, area: f64, aspects: &HashMap<u32, f64>, out: &mut Vec<f64>) {
    match node {
        Slot::Leaf { .. } => out.push(area),
        Slot::Cut { vertical, left, right } => {
            let (al, ar) = (aspect(left, aspects), aspect(right, aspects));
            let (frac_left, frac_right) = if *vertical {
                (al / (al + ar), ar / (al + ar))
            } else {
                (ar / (al + ar), al / (al + ar))
            };
            leaf_areas(left, area * frac_left, aspects, out);
            leaf_areas(right, area * frac_right, aspects, out);
        }
    }
}

pub fn mutate(indiv: &mut Individual, pool: &[u32], min_images: usize, max_images: usize, allow_rotate: bool, rng: &mut impl Rng) {
    let leaves = leaf_count(&indiv.tree);
    let unused = unused_ids(&indiv.tree, pool);

    // Build the set of moves valid for the current tree, then pick one.
    let mut moves: Vec<u8> = vec![0]; // 0 = flip a cut (always valid for >1 leaf, no-op otherwise)
    if leaves >= 2 {
        moves.push(1); // swap two images
    }
    if !unused.is_empty() {
        moves.push(2); // replace an image with an unused one
        if leaves < max_images {
            moves.push(3); // grow: split a leaf into two
        }
    }
    if leaves > min_images && leaves > 1 {
        moves.push(4); // shrink: collapse a cut
    }
    if allow_rotate {
        moves.push(5); // toggle a leaf's 90-degree rotation
    }

    match *moves.choose(rng).unwrap() {
        0 => flip_random_cut(&mut indiv.tree, rng),
        1 => {
            let mut ids = Vec::new();
            collect_ids(&indiv.tree, &mut ids);
            let (i, j) = (rng.gen_range(0..ids.len()), rng.gen_range(0..ids.len()));
            ids.swap(i, j);
            write_ids(&mut indiv.tree, &mut ids.into_iter());
        }
        2 => {
            let mut ids = Vec::new();
            collect_ids(&indiv.tree, &mut ids);
            let i = rng.gen_range(0..ids.len());
            ids[i] = *unused.choose(rng).unwrap();
            write_ids(&mut indiv.tree, &mut ids.into_iter());
        }
        3 => {
            let target = rng.gen_range(0..leaves);
            let new_id = *unused.choose(rng).unwrap();
            let vertical = rng.gen::<bool>();
            grow_nth_leaf(&mut indiv.tree, target, &mut 0, new_id, vertical);
        }
        4 => {
            let cuts = count_cuts(&indiv.tree);
            let target = rng.gen_range(0..cuts);
            let keep_left = rng.gen::<bool>();
            shrink_nth_cut(&mut indiv.tree, target, &mut 0, keep_left);
        }
        5 => {
            let target = rng.gen_range(0..leaves);
            toggle_nth_leaf(&mut indiv.tree, target, &mut 0);
        }
        _ => unreachable!(),
    }
}

fn unused_ids(tree: &Slot, pool: &[u32]) -> Vec<u32> {
    let mut used = Vec::new();
    collect_ids(tree, &mut used);
    let used: HashSet<u32> = used.into_iter().collect();
    pool.iter().copied().filter(|id| !used.contains(id)).collect()
}

fn write_ids(node: &mut Slot, ids: &mut impl Iterator<Item = u32>) {
    match node {
        Slot::Leaf { id, .. } => *id = ids.next().unwrap(),
        Slot::Cut { left, right, .. } => {
            write_ids(left, ids);
            write_ids(right, ids);
        }
    }
}

fn count_cuts(node: &Slot) -> usize {
    match node {
        Slot::Leaf { .. } => 0,
        Slot::Cut { left, right, .. } => 1 + count_cuts(left) + count_cuts(right),
    }
}

fn toggle_nth_leaf(node: &mut Slot, target: usize, counter: &mut usize) -> bool {
    match node {
        Slot::Leaf { rotated, .. } => {
            if *counter == target {
                *rotated = !*rotated;
                return true;
            }
            *counter += 1;
            false
        }
        Slot::Cut { left, right, .. } => {
            toggle_nth_leaf(left, target, counter) || toggle_nth_leaf(right, target, counter)
        }
    }
}

fn flip_random_cut(node: &mut Slot, rng: &mut impl Rng) {
    let cuts = count_cuts(node);
    if cuts == 0 {
        return;
    }
    flip_nth_cut(node, rng.gen_range(0..cuts), &mut 0);
}

fn flip_nth_cut(node: &mut Slot, target: usize, counter: &mut usize) -> bool {
    if let Slot::Cut { vertical, left, right } = node {
        if *counter == target {
            *vertical = !*vertical;
            return true;
        }
        *counter += 1;
        return flip_nth_cut(left, target, counter) || flip_nth_cut(right, target, counter);
    }
    false
}

fn grow_nth_leaf(node: &mut Slot, target: usize, counter: &mut usize, new_id: u32, vertical: bool) -> bool {
    match node {
        Slot::Leaf { id, rotated } => {
            if *counter == target {
                let (old, old_rotated) = (*id, *rotated);
                *node = Slot::Cut {
                    vertical,
                    left: Box::new(Slot::Leaf { id: old, rotated: old_rotated }),
                    right: Box::new(Slot::Leaf { id: new_id, rotated: false }),
                };
                return true;
            }
            *counter += 1;
            false
        }
        Slot::Cut { left, right, .. } => {
            grow_nth_leaf(left, target, counter, new_id, vertical)
                || grow_nth_leaf(right, target, counter, new_id, vertical)
        }
    }
}

fn shrink_nth_cut(node: &mut Slot, target: usize, counter: &mut usize, keep_left: bool) -> bool {
    if let Slot::Cut { left, right, .. } = node {
        if *counter == target {
            let kept = if keep_left { left.as_mut() } else { right.as_mut() };
            *node = std::mem::replace(kept, Slot::Leaf { id: 0, rotated: false });
            return true;
        }
        *counter += 1;
        return shrink_nth_cut(left, target, counter, keep_left)
            || shrink_nth_cut(right, target, counter, keep_left);
    }
    false
}
