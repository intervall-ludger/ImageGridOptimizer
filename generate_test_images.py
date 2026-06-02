# /// script
# dependencies = ["pillow"]
# ///
import argparse
import colorsys
import logging
import random
from pathlib import Path

from PIL import Image, ImageDraw

logging.basicConfig(level=logging.INFO, format="%(message)s")
log = logging.getLogger("gen")

# Difficulty = aspect-ratio variety. Easy tiles into a clean grid, hard forces
# the slicing tree to interlock wildly different shapes.
LEVELS: dict[str, list[tuple[int, int]]] = {
    "easy": [(1, 1)],
    "medium": [(1, 1), (4, 3), (3, 4), (3, 2), (2, 3)],
    "hard": [
        (1, 1), (16, 9), (9, 16), (4, 1), (1, 4), (3, 1), (1, 3), (2, 1), (1, 2),
        (5, 2), (2, 5), (7, 3), (3, 7), (8, 1), (1, 8), (5, 4), (4, 5), (21, 9),
        (3, 5), (5, 3), (6, 1), (1, 6), (10, 3), (3, 10),
    ],
}


def make_image(width: int, height: int, hue: float, label: str) -> Image.Image:
    r, g, b = (int(c * 255) for c in colorsys.hsv_to_rgb(hue, 0.55, 0.9))
    img = Image.new("RGB", (width, height), (r, g, b))
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, width - 1, height - 1], outline=(30, 30, 30), width=max(2, min(width, height) // 80))
    draw.text((8, 8), label, fill=(20, 20, 20))
    return img


def generate_level(level: str, count: int, base_size: int, out_dir: Path, rng: random.Random) -> None:
    ratios = LEVELS[level]
    out_dir.mkdir(parents=True, exist_ok=True)
    for i in range(count):
        rw, rh = rng.choice(ratios)
        scale = rng.uniform(0.7, 1.4)
        width = max(40, int(base_size * scale * (rw / max(rw, rh)) ** 0.5 * (rw / rh) ** 0.5))
        height = max(40, int(width * rh / rw))
        hue = (i / count + rng.uniform(-0.03, 0.03)) % 1.0
        img = make_image(width, height, hue, f"{width}x{height}")
        img.save(out_dir / f"img_{i:02d}_{width}x{height}.png")
    log.info("%s: %d images (%d aspect ratios) -> %s", level, count, len(ratios), out_dir)


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate graded-difficulty test images (aspect-ratio variety).")
    parser.add_argument("--out", type=Path, default=Path("_test_dir"), help="Base output directory.")
    parser.add_argument("--count", type=int, default=18, help="Images per difficulty level.")
    parser.add_argument("--base-size", type=int, default=520, help="Reference size in pixels.")
    parser.add_argument("--seed", type=int, default=42, help="Random seed for reproducibility.")
    args = parser.parse_args()

    rng = random.Random(args.seed)
    for level in LEVELS:
        generate_level(level, args.count, args.base_size, args.out / level, rng)


if __name__ == "__main__":
    main()
