# ImageGridOptimizer

ImageGridOptimizer is a command-line tool designed to optimize the placement of images from a directory into a single collage. It intelligently arranges images by checking for empty spaces and placing images in a way that minimizes the overall size of the collage. Additionally, it provides an option to filter images based on their extensions or parts of their filenames.

## Features

- **Optimized Placement**: Efficiently places images in a collage to minimize the overall size.
- **Filtering**: Allows filtering of images based on extensions or parts of filenames.
- **White Border Addition**: Adds a white border around each image for better distinction in the collage.

## Installation

1. Ensure you have Rust and Cargo installed on your machine.
2. Clone this repository:
   ```bash
   git clone https://github.com/ludgerradke/ImageGridOptimizer.git
   ```
3. Navigate to the project directory and build the project:
   ```bash
   cd ImageGridOptimizer
   cargo build --release
   ```

## Usage

Navigate to the project's target/release directory and run:

```bash
./ImageGridOptimizer [DIRECTORY] -f [FILTER]
```

- `DIRECTORY`: The directory containing the images you want to optimize.
- `FILTER` (optional): Filter images by extension (e.g., `.jpg`, `.png`) or part of the filename (e.g., `*img_1*`).

The optimized collage will be saved as `output.jpg` in the project's root directory.

## Example

To optimize images in the directory `my_images` with a `.jpg` extension:

```bash
./ImageGridOptimizer my_images -f .jpg
```

## Contributing

Contributions are welcome! Please fork the repository and create a pull request with your changes.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
