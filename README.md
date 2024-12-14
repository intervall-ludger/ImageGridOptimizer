# ImageGridOptimizer

ImageGridOptimizer is a command-line utility that uses a Genetic Algorithm (GA) to arrange multiple images from a specified directory into a cohesive collage. It can efficiently handle a large number of trials (e.g., millions) and leverages parallel processing (via Rayon) to speed up computation. Images are arranged to minimize the collage dimensions, filtered according to user criteria, and presented with a white border to enhance visual separation.

**Still in Progress**

### Open Issues
- [ ] Support additional image formats such as TIFF or other non-natively supported formats.

---

## Key Features

- **Large-Scale Trials & Parallelization**: Configurable to run millions of trials, utilizing multiple CPU cores to accelerate the genetic algorithm.
- **Genetic Algorithm Optimization**: Employs a GA to find an optimal image layout, considering fitness factors such as aspect ratio and minimal whitespace.
- **Flexible Image Filtering**: Easily filter images by extension or filename substring.
- **Automatic Borders & Centering**: Adds a white border around each image and centers the final layout to distribute free space more evenly.

## Getting Started

### Prerequisites

- **Rust and Cargo**:  
  Install from [Rustup](https://rustup.rs/).

- **Rayon**:  
  Dependency is managed automatically by Cargo.

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/ludgerradke/ImageGridOptimizer.git
   ```
2. Navigate to the project directory:
   ```bash
   cd ImageGridOptimizer
   ```
3. Build the project in release mode:
   ```bash
   cargo build --release
   ```

### How to Use

Run the program from the `target/release` directory (or specify the full path):

```bash
./ImageGridOptimizer [DIRECTORY] [OPTIONS]
```

**Key Options:**

- `-f, --filter <FILTER>`  
  Filters images (e.g., by extension `.jpg` or substring `img_`).

- `-w, --width <WIDTH>`  
  Scales all images to a specified width, preserving aspect ratio.

- `--pop-size <POP_SIZE>`  
  Population size for the GA (default: 1000).

- `--gens <GENS>`  
  Number of generations (default: 3000).

- `--min-images <MIN_IMAGES>`  
  Minimum number of images per collage.

- `--max-images <MAX_IMAGES>`  
  Maximum number of images per collage.

- `--mutation-rate <MUTATION_RATE>`  
  Mutation rate for the GA.

- `--crossover-rate <CROSSOVER_RATE>`  
  Crossover rate for the GA.

**Example:**

```bash
./ImageGridOptimizer my_photos -f .jpg -w 800 --pop-size 1000 --gens 3000 --min-images 20 --max-images 70 --mutation-rate 0.1 --crossover-rate 0.7
```

This example:
- Loads `.jpg` images from `my_photos`
- Scales them to a width of 800 pixels
- Runs the GA with a population of 1000 and 3000 generations (~3 million trials)
- Uses between 20 and 70 images per collage
- Applies a mutation rate of 0.1 and a crossover rate of 0.7
- Saves the final collage as `output.jpg` in the current directory

## Example Output

For a simpler test, consider a smaller run:

```bash
.\target\release\ImageGridOptimizer .\_test_dir\ --min-images 10 --max-images 20
```

This will create a collage using between 10 and 20 images from the `test_images` directory. The result look like:

![Collage Example](output.jpg)


## Contributing

Contributions are welcome!
- Fork the repository
- Create a new branch for your feature or bugfix
- Open a pull request

## Releasing a Version

To create a tagged release, for example version `v1.0`:

```bash
git tag v1.0
git push origin v1.0
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.