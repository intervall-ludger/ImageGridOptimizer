//! Main entry point for ImageGridOptimizer.

mod cli;
mod image_processor;

fn main() {
    // Parses command-line arguments including directory, filter, standard width, and additional parameters for collage creation.
    let (dir, filter, standard_width, num_trials, min_images, max_images) = cli::parse_args();

    // Calls the process_images function with the parsed arguments and captures the output image.
    let output_image = image_processor::process_images(&dir, filter, standard_width, num_trials, min_images, max_images);

    // Saves the output image to a file.
    output_image.save("output.jpg").unwrap();
}
