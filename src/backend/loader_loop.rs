use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use eframe::egui::ColorImage;
use image::RgbaImage;
use rodio::{Decoder, Source};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey, Visual};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

use crate::backend::loader_messages::{Request, Response};
use crate::image_utils;
use crate::track_metadata::TrackMetaData;

pub fn run(request_receiver: Receiver<Request>, response_sender: Sender<Response>) {
    loop {
        match request_receiver.recv() {
            Ok(req) => match req {
                Request::Track(path) => {
                    println!("Loader: load request received: {path:?}");
                    handle_request(path, &response_sender)
                }
            },
            Err(e) => {
                println!("Error in loader thread: {e:?}");
                exit(1);
            }
        }
    }
}

fn handle_request(path: PathBuf, response_sender: &Sender<Response>) {
    // metadata
    let file = File::open(&path).unwrap();
    let source = Decoder::new(file).unwrap();
    let duration = source.total_duration();
    let mut metadata = match get_track_metadata(&path) {
        None => {
            let mut m = TrackMetaData::default();
            if let Some(name) = path.file_name() {
                if let Some(name) = name.to_str() {
                    m.name = name.to_string();
                }
            }
            m
        }
        Some(m) => m,
    };
    metadata.duration = duration;
    let metadata = Arc::new(metadata);

    // file (again)
    let file = File::open(&path).unwrap();

    response_sender
        .send(Response::Track(file, metadata))
        .unwrap();
    println!("Loader: Load response sent ({path:?})");
}

pub fn get_track_metadata(path: &Path) -> Option<TrackMetaData> {
    let file = File::open(path).ok()?;

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

    let mut track = TrackMetaData::default();

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
        track.image = get_color_image_from_track_path(path);
    }

    Some(track)
}

fn get_color_image_from_visual(v: &Visual) -> Option<ColorImage> {
    let data_box = &*v.data;
    let image = get_rgba_image_from_slice(data_box)?;
    let image = image_utils::get_color_image_from_rgba_image(image);
    Some(image)
}

fn get_rgba_image_from_slice(data: &[u8]) -> Option<RgbaImage> {
    let image = image::load_from_memory(data).ok()?;
    Some(image.to_rgba8())
}

fn get_color_image_from_track_path(path: &Path) -> Option<ColorImage> {
    let parent = path.parent()?;
    ["cover.jpg", "cover.png"] // try each option until one works
        .iter()
        .find_map(|file_name| image_utils::load_color_image(&parent.join(file_name)))
}
