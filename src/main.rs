use image::{imageops, Rgb, RgbImage};

fn main() {
    let mut img = RgbImage::new(128, 128);

    for x in 0..=10 {
        for y in 0..=10 {
            img.put_pixel(x, y, Rgb([255, 255, 255]));
        }
    }

    imageops::flip_vertical_in_place(&mut img);
    img.save("output/empty.tga").unwrap();
}
