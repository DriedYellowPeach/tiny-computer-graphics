/// Part 1: understandable raytracing
/// Step 1: write an image to the disk
/// save the picture to disk
use image::{imageops, Rgb, RgbImage};

pub fn render_example_scene() {
    let mut img = RgbImage::new(256, 256);

    for x in 0..img.width() {
        for y in 0..img.height() {
            let red = 255 * x / img.width();
            let green = 255 * y / img.height();

            img.put_pixel(x, y, Rgb([red as u8, green as u8, 0]));
        }
    }

    imageops::flip_vertical_in_place(&mut img);
    img.save("output/ray_tracing_step_one_scene.tga").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_example_scene() {
        render_example_scene();
    }
}
