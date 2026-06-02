use clap::{App, Arg};

pub struct Config {
    pub dir: String,
    pub filter: Option<String>,
    pub standard_width: Option<u32>,
    pub population_size: usize,
    pub generations: usize,
    pub min_images: usize,
    pub max_images: usize,
    pub mutation_rate: f64,
    pub target_aspect: f64,
    pub flex: f64,
    pub gutter: u32,
    pub margin: u32,
    pub width: u32,
    pub allow_rotate: bool,
}

pub fn parse_args() -> Config {
    let matches = App::new("ImageGridOptimizer GA")
        .version("3.0")
        .author("Ludger Radke")
        .about("Arranges images into a gap-free collage using a slicing-tree genetic algorithm.")
        .arg(Arg::with_name("DIRECTORY").help("Directory containing the images.").required(true).index(1))
        .arg(Arg::with_name("filter").short("f").long("filter").value_name("FILTER")
            .help("Filter for images (extension or part of filename).").takes_value(true))
        .arg(Arg::with_name("standard_width").short("w").long("std-width").value_name("WIDTH")
            .help("Optional standard width images are downscaled to before packing.").takes_value(true))
        .arg(Arg::with_name("population_size").long("pop-size").value_name("POP_SIZE")
            .help("Population size for the genetic algorithm.").takes_value(true))
        .arg(Arg::with_name("generations").long("gens").value_name("GENS")
            .help("Number of generations for the genetic algorithm.").takes_value(true))
        .arg(Arg::with_name("min_images").long("min-images").value_name("MIN_IMAGES")
            .help("Minimum number of images per collage.").takes_value(true))
        .arg(Arg::with_name("max_images").long("max-images").value_name("MAX_IMAGES")
            .help("Maximum number of images per collage.").takes_value(true))
        .arg(Arg::with_name("mutation_rate").long("mutation-rate").value_name("MUTATION_RATE")
            .help("Probability of applying extra mutations to an offspring.").takes_value(true))
        .arg(Arg::with_name("aspect").long("aspect").value_name("ASPECT")
            .help("Target aspect ratio (width/height) of the whole collage.").takes_value(true))
        .arg(Arg::with_name("flex").long("flex").value_name("FLEX")
            .help("0..1: 1 fills cells gap-free, 0 keeps native sizes with gaps.").takes_value(true))
        .arg(Arg::with_name("gutter").long("gutter").value_name("GUTTER")
            .help("White spacing in pixels between images.").takes_value(true))
        .arg(Arg::with_name("margin").long("margin").value_name("MARGIN")
            .help("White border in pixels around the collage.").takes_value(true))
        .arg(Arg::with_name("out_width").long("width").value_name("OUT_WIDTH")
            .help("Content width in pixels of the rendered collage.").takes_value(true))
        .arg(Arg::with_name("rotate").long("rotate")
            .help("Allow images to be rotated 90 degrees for tighter packing."))
        .get_matches();

    Config {
        dir: matches.value_of("DIRECTORY").unwrap().to_string(),
        filter: matches.value_of("filter").map(|s| s.to_string()),
        standard_width: matches.value_of("standard_width").map(|w| w.parse().expect("Invalid std-width")),
        population_size: matches.value_of("population_size").unwrap_or("500").parse().expect("Invalid population size"),
        generations: matches.value_of("generations").unwrap_or("600").parse().expect("Invalid number of generations"),
        min_images: matches.value_of("min_images").unwrap_or("6").parse().expect("Invalid min_images"),
        max_images: matches.value_of("max_images").unwrap_or("60").parse().expect("Invalid max_images"),
        mutation_rate: matches.value_of("mutation_rate").unwrap_or("0.3").parse().expect("Invalid mutation rate"),
        target_aspect: matches.value_of("aspect").unwrap_or("1.0").parse().expect("Invalid aspect"),
        flex: matches.value_of("flex").unwrap_or("1.0").parse().expect("Invalid flex"),
        gutter: matches.value_of("gutter").unwrap_or("8").parse().expect("Invalid gutter"),
        margin: matches.value_of("margin").unwrap_or("12").parse().expect("Invalid margin"),
        width: matches.value_of("out_width").unwrap_or("1600").parse().expect("Invalid width"),
        allow_rotate: matches.is_present("rotate"),
    }
}
