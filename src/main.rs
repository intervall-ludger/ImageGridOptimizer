//! Main entry point for ImageGridOptimizer.

mod cli;
mod image_processor;

fn main() {
    let (dir, filter, standard_width) = cli::parse_args();
    let output_image = image_processor::process_images(&dir, filter, standard_width);
    output_image.save("output.jpg").unwrap();
}
