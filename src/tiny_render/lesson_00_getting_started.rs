use image::{imageops, Rgb, RgbImage};

pub fn draw_a_rectangular() {
    let mut img = RgbImage::new(128, 128);

    for x in 0..=10 {
        for y in 0..=10 {
            img.put_pixel(x, y, Rgb([255, 255, 255]));
            // img.put_pixel(y, x, Rgb([255, 0, 0]));
        }
    }

    imageops::flip_vertical_in_place(&mut img);
    img.save("output/rect.tga").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_a_rectangular() {
        draw_a_rectangular();
    }
}
