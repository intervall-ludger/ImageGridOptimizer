//! Handles command-line interface parsing for ImageGridOptimizer.

use clap::{App, Arg};

/// Parses command line arguments and returns the parameters for image processing.
///
/// # Returns
///
/// A tuple containing:
/// - The directory as a `String`
/// - An `Option<String>` for the filter
/// - An `Option<u32>` for the standard width
/// - `usize` for the number of trials
/// - `usize` for the minimum number of images per collage
/// - `usize` for the maximum number of images per collage
pub fn parse_args() -> (String, Option<String>, Option<u32>, usize, usize, usize) {
    let matches = App::new("Image Optimizer")
        .version("1.0")
        .author("Ludger Radke")
        .about("Optimizes image placement from a directory by creating a collage.")
        .arg(Arg::with_name("DIRECTORY")
            .help("The directory containing the images.")
            .required(true)
            .index(1))
        .arg(Arg::with_name("filter")
            .short("f")
            .long("filter")
            .value_name("FILTER")
            .help("Filter images by extension (e.g., .jpg, .png) or part of the filename (e.g., *img_1*).")
            .takes_value(true))
        .arg(Arg::with_name("standard_width")
            .short("w")
            .long("width")
            .value_name("WIDTH")
            .help("Optional standard width to scale the images to.")
            .takes_value(true))
        .arg(Arg::with_name("num_trials")
            .short("n")
            .long("num-trials")
            .value_name("NUM_TRIALS")
            .help("The number of trials to generate collages.")
            .takes_value(true))
        .arg(Arg::with_name("min_images")
            .long("min-images")
            .value_name("MIN_IMAGES")
            .help("The minimum number of images per collage.")
            .takes_value(true))
        .arg(Arg::with_name("max_images")
            .long("max-images")
            .value_name("MAX_IMAGES")
            .help("The maximum number of images per collage.")
            .takes_value(true))
        .get_matches();

    let dir = matches.value_of("DIRECTORY").unwrap().to_string();
    let filter = matches.value_of("filter").map(|s| s.to_string());
    let standard_width = matches
        .value_of("standard_width")
        .map(|w| w.parse::<u32>().expect("Invalid width value"));
    let num_trials = matches
        .value_of("num_trials")
        .unwrap_or("3000")
        .parse::<usize>()
        .expect("Invalid number of trials");
    let min_images = matches
        .value_of("min_images")
        .unwrap_or("20")
        .parse::<usize>()
        .expect("Invalid minimum number of images");
    let max_images = matches
        .value_of("max_images")
        .unwrap_or("70")
        .parse::<usize>()
        .expect("Invalid maximum number of images");

    (dir, filter, standard_width, num_trials, min_images, max_images)
}
