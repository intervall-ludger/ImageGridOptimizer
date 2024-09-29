//! Main entry point for the ImageGridOptimizer.

mod cli;
mod image_processor;

fn main() {
    // Parse command-line arguments.
    let (dir, filter, standard_width, num_trials, min_images, max_images) = cli::parse_args();
    println!("Arguments parsed:");
    println!("Directory: {}", dir);
    println!("Filter: {:?}", filter);
    println!("Standard Width: {:?}", standard_width);
    println!("Number of Trials: {}", num_trials);
    println!("Minimum Images: {}", min_images);
    println!("Maximum Images: {}", max_images);

    // Process images and create the collage.
    println!("Processing images...");
    let output_image = image_processor::process_images(
        &dir,
        filter,
        standard_width,
        num_trials,
        min_images,
        max_images,
    );
    println!("Images processed.");

    // Save the output image to a file.
    println!("Saving output image to 'output.jpg'...");
    match output_image.save("output.jpg") {
        Ok(_) => println!("Image successfully saved."),
        Err(e) => eprintln!("Error saving image: {}", e),
    }
}
