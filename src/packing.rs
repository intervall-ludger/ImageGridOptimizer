use std::collections::HashMap;
use image::{DynamicImage, GenericImageView};
use rect_packer::{Config, Packer, Rect};

pub const DESIRED_ASPECT_RATIO: f64 = 1.0;
const PADDING_SIZE: u32 = 5;

pub fn pack_images(
    image_ids: &Vec<u32>,
    image_map: &HashMap<u32, DynamicImage>,
) -> (Vec<(u32, Rect)>, u32, u32) {
    if image_ids.is_empty() {
        return (vec![], 0, 0);
    }

    let total_area: u64 = image_ids.iter().map(|id| {
        let img = image_map.get(id).unwrap();
        let (w, h) = img.dimensions();
        (w as u64) * (h as u64)
    }).sum();

    let estimated_height = ((total_area as f64 / DESIRED_ASPECT_RATIO).sqrt()) as u32;
    let estimated_width = (DESIRED_ASPECT_RATIO * estimated_height as f64) as u32;

    let mut scale_factor = 1.0;
    let max_attempts = 5;
    for _attempt in 0..max_attempts {
        let pack_w = (estimated_width as f64 * scale_factor) as i32;
        let pack_h = (estimated_height as f64 * scale_factor) as i32;

        let config = Config {
            width: pack_w,
            height: pack_h,
            border_padding: 0,
            rectangle_padding: PADDING_SIZE as i32,
        };

        let mut packer = Packer::new(config);
        let mut packed_locations = Vec::new();
        let mut max_width = 0;
        let mut max_height = 0;

        let mut all_fit = true;
        for id in image_ids {
            let img = image_map.get(id).unwrap();
            let (w, h) = img.dimensions();
            if let Some(rect) = packer.pack(w as i32, h as i32, false) {
                packed_locations.push((*id, rect));
                if (rect.x + rect.width) as u32 > max_width {
                    max_width = (rect.x + rect.width) as u32;
                }
                if (rect.y + rect.height) as u32 > max_height {
                    max_height = (rect.y + rect.height) as u32;
                }
            } else {
                all_fit = false;
                break;
            }
        }

        if all_fit {
            return (packed_locations, max_width, max_height);
        }

        scale_factor *= 1.2;
    }

    (vec![], 0, 0)
}
