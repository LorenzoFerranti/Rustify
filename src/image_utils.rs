use std::path::{Path, PathBuf};
use eframe::egui::{Color32, ColorImage};
use image::RgbaImage;


pub fn load_color_image(path: &Path) -> Option<ColorImage> {
    let path_buf = PathBuf::from(path);
    let dyn_img = image::open(&path_buf).ok()?;
    Some(get_color_image_from_rgba_image(dyn_img.to_rgba8()))
}

pub fn get_color_image_from_rgba_image(image: RgbaImage) -> ColorImage {
    let (width, height) = image.dimensions();
    let pixels: Vec<_> = image
        .pixels()
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    ColorImage {
        size: [width as usize, height as usize],
        pixels,
    }
}