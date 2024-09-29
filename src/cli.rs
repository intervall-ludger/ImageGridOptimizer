//! Handles parsing of command-line arguments for the ImageGridOptimizer.

use clap::{App, Arg};

/// Parses command-line arguments and returns parameters for image processing.
///
/// # Returns
///
/// A tuple containing:
/// - Directory as `String`
/// - `Option<String>` for the filter
/// - `Option<u32>` for the standard width
/// - `usize` for the number of trials
/// - `usize` for the minimum number of images per collage
/// - `usize` for the maximum number of images per collage
pub fn parse_args() -> (String, Option<String>, Option<u32>, usize, usize, usize) {
    let matches = App::new("Image Optimizer")
        .version("1.0")
        .author("Ludger Radke")
        .about("Optimizes image arrangement from a directory by creating a collage.")
        .arg(
            Arg::with_name("DIRECTORY")
                .help("The directory containing the images.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("filter")
                .short("f")
                .long("filter")
                .value_name("FILTER")
                .help("Filter images by extension (e.g., .jpg, .png) or part of the filename (e.g., *img_1*).")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("standard_width")
                .short("w")
                .long("width")
                .value_name("WIDTH")
                .help("Optional standard width to which images should be scaled.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("num_trials")
                .short("n")
                .long("num-trials")
                .value_name("NUM_TRIALS")
                .help("Number of trials to generate collages.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("min_images")
                .long("min-images")
                .value_name("MIN_IMAGES")
                .help("Minimum number of images per collage.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("max_images")
                .long("max-images")
                .value_name("MAX_IMAGES")
                .help("Maximum number of images per collage.")
                .takes_value(true),
        )
        .get_matches();

    let dir = matches.value_of("DIRECTORY").unwrap().to_string();
    let filter = matches.value_of("filter").map(|s| s.to_string());
    let standard_width = matches
        .value_of("standard_width")
        .map(|w| w.parse::<u32>().expect("Invalid value for width"));
    let num_trials = matches
        .value_of("num_trials")
        .unwrap_or("100000") // Default to 10,000 trials
        .parse::<usize>()
        .expect("Invalid number of trials");
    let min_images = matches
        .value_of("min_images")
        .unwrap_or("20")
        .parse::<usize>()
        .expect("Invalid minimum number of images");
    let max_images = matches
        .value_of("max_images")
        .unwrap_or("50")
        .parse::<usize>()
        .expect("Invalid maximum number of images");

    (dir, filter, standard_width, num_trials, min_images, max_images)
}
