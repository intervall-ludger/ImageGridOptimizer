//! Main entry point for ImageGridOptimizer.

mod cli;
mod image_processor;

fn main() {
    let (dir, filter) = cli::parse_args();
    let output_image = image_processor::process_images(&dir, filter);
    output_image.save("output.jpg").unwrap();
}
