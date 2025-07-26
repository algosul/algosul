use std::fmt::Write;

use algosul_derive::from_dir;
use image::{imageops::FilterType, Rgb};
from_dir!(pub mod assets from "rc" {
    text [include ["lang/*.toml"] exclude []];
    binary [include ["images/*.png"] exclude []];
});
#[test]
fn main() {
    let locale = assets::lang::en_US;
    println!("{locale}");
    let image = image::load_from_memory(assets::images::_0).unwrap();
    let image = image.resize(80, 60, FilterType::Lanczos3);
    let image = image.to_rgb8();
    let (width, height) = image.dimensions();
    let mut buffer = String::new();
    for y in (0..height).step_by(2) {
        for x in 0..width {
            let top = image.get_pixel(x, y);
            let bottom = if y + 1 < height {
                image.get_pixel(x, y + 1)
            } else {
                &Rgb([0, 0, 0])
            };
            write!(
                buffer,
                "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m▀",
                top[0], top[1], top[2], bottom[0], bottom[1], bottom[2]
            )
            .unwrap();
        }
        writeln!(buffer, "\x1b[0m").unwrap();
    }
    println!("{buffer}");
}
