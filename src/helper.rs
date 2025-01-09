use crate::sink_wrapper::Track;

use eframe::egui::{Color32, ColorImage};
use std::fs::{DirEntry, File};
use std::time::Duration;
use image::RgbaImage;

use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey, Visual};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

pub fn get_track(entry: &DirEntry) -> Option<Track> {
    let path_buf = entry.path();
    let file = File::open(path_buf).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let probe = get_probe();
    let hint = Hint::new();

    // probe file
    let mut probed = probe
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe format: {}", e))
        .ok()?;

    // get metadata
    let binding = probed.metadata.get()?;
    let current_metadata = binding.current()?;
    let mut track = Track::default();

    // read tags
    for tag in current_metadata.tags() {
        if let Some(std_key) = tag.std_key {
            let value = tag.value.to_string();
            match std_key {
                StandardTagKey::Album => track.album = value,
                StandardTagKey::Artist => track.artist = value,
                StandardTagKey::TrackTitle => track.name = value,
                _ => {}
            }
        }
    }

    // read cover image
    if let Some(v) = current_metadata.visuals().first() {
        track.image = get_color_image_from_visual(v);
    } else {
        track.image = get_color_image_from_directory(entry);
        if let Some(_) = track.image {
            println!("GOT IMAGE FROM DIR!!!!")
        }
    }

    Some(track)
}

fn get_color_image_from_visual(v: &Visual) -> Option<ColorImage> {
    let data_box = &*v.data;
    let image = get_rgba_image_from_slice(data_box)?;
    let image = get_color_image_from_rgba_image(image);
    Some(image)
}

fn get_rgba_image_from_slice(data: &[u8]) -> Option<RgbaImage> {
    let image = image::load_from_memory(data).ok()?;
    Some(image.to_rgba8())
}

fn get_color_image_from_directory(entry: &DirEntry) -> Option<ColorImage> {
    let mut path = entry.path();
    path.pop();
    path.push("cover.jpg");
    println!("{:?}", path);
    let mut dyn_img = image::open(&path).ok();
    match dyn_img {
        None => {
            println!("NONE");
            path.pop();
            path.push("cover.png");
            let rgba_img = image::open(&path).ok()?.to_rgba8();
            Some(get_color_image_from_rgba_image(rgba_img))
        }
        Some(di) => {
            println!("SOME");
            Some(get_color_image_from_rgba_image(di.to_rgba8()))
        }
    }
}

fn get_color_image_from_rgba_image(image: RgbaImage) -> ColorImage {
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

pub fn formatted_duration(d: &Duration) -> String {
    let tot = d.as_secs();
    let sec = tot % 60;
    let min = tot / 60;
    let mut sec_padding = "".to_string();
    let mut min_padding = "".to_string();
    if sec < 10 {
        sec_padding.push('0');
    }
    if min < 10 {
        min_padding.push('0');
    }
    format!("{}{}:{}{}", min_padding, min, sec_padding, sec)

}
