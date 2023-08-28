//! Handles command-line interface parsing for ImageGridOptimizer.

use clap::{Arg, App};

/// Parses command line arguments and returns directory and filter.
///
/// # Returns
///
/// A tuple containing the directory as a `String` and an `Option<String>` for the filter.
pub fn parse_args() -> (String, Option<String>) {
    let matches = App::new("Image Optimizer")
        .version("1.0")
        .author("Ludger Radke")
        .about("Optimizes image placement from a directory")
        .arg(Arg::with_name("DIRECTORY")
            .help("The directory containing the images")
            .required(true)
            .index(1))
        .arg(Arg::with_name("filter")
            .short("f")
            .long("filter")
            .value_name("FILTER")
            .help("Filter images by extension (e.g. .jpg, .png) or part of the filename (e.g. *img_1*)")
            .takes_value(true))
        .get_matches();

    let dir = matches.value_of("DIRECTORY").unwrap().to_string();
    let filter = matches.value_of("filter").map(|s| s.to_string());

    (dir, filter)
}
