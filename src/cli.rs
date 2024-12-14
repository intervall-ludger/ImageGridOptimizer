use clap::{App, Arg};

pub fn parse_args() -> (String, Option<String>, Option<u32>, usize, usize, usize, usize, f64, f64) {
    let matches = App::new("ImageGridOptimizer GA")
        .version("1.0")
        .author("Senior Developer")
        .about("Optimizes the arrangement of images using a Genetic Algorithm.")
        .arg(
            Arg::with_name("DIRECTORY")
                .help("Directory containing the images.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("filter")
                .short("f")
                .long("filter")
                .value_name("FILTER")
                .help("Filter for images (extension or part of filename).")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("standard_width")
                .short("w")
                .long("width")
                .value_name("WIDTH")
                .help("Optional standard width for scaling images.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("population_size")
                .long("pop-size")
                .value_name("POP_SIZE")
                .help("Population size for the genetic algorithm.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("generations")
                .long("gens")
                .value_name("GENS")
                .help("Number of generations for the genetic algorithm.")
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
        .arg(
            Arg::with_name("mutation_rate")
                .long("mutation-rate")
                .value_name("MUTATION_RATE")
                .help("Mutation rate for the genetic algorithm.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("crossover_rate")
                .long("crossover-rate")
                .value_name("CROSSOVER_RATE")
                .help("Crossover rate for the genetic algorithm.")
                .takes_value(true),
        )
        .get_matches();

    let dir = matches.value_of("DIRECTORY").unwrap().to_string();
    let filter = matches.value_of("filter").map(|s| s.to_string());
    let standard_width = matches
        .value_of("standard_width")
        .map(|w| w.parse::<u32>().expect("Invalid width"));

    // Default large values to handle large number of trials
    let population_size = matches.value_of("pop_size").unwrap_or("1000").parse::<usize>().expect("Invalid population size");
    let generations = matches.value_of("gens").unwrap_or("3000").parse::<usize>().expect("Invalid number of generations");
    let min_images = matches.value_of("min_images").unwrap_or("6").parse::<usize>().expect("Invalid min_images");
    let max_images = matches.value_of("max_images").unwrap_or("60").parse::<usize>().expect("Invalid max_images");
    let mutation_rate = matches.value_of("mutation_rate").unwrap_or("0.1").parse::<f64>().expect("Invalid mutation rate");
    let crossover_rate = matches.value_of("crossover_rate").unwrap_or("0.7").parse::<f64>().expect("Invalid crossover rate");

    (dir, filter, standard_width, population_size, generations, min_images, max_images, mutation_rate, crossover_rate)
}
