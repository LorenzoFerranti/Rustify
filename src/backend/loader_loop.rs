use std::cmp::PartialEq;
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

#[derive(PartialEq, Eq)]
enum State {
    Active,
    WaitingForReset,
}

pub fn run(request_receiver: Receiver<Request>, response_sender: Sender<Response>) {
    let mut state = State::Active;
    loop {
        match request_receiver.recv() {
            Ok(req) => match req {
                Request::Track(path) => {
                    //println!("Loader: load request received: {}", path.display());
                    handle_request(&mut state, path, &response_sender)
                }
                Request::ResetCompleted => {
                    state = State::Active;
                }
            },
            Err(e) => {
                println!("Error in loader thread: {e:?}");
                exit(1);
            }
        }
    }
}


fn handle_request(state: &mut State, path: PathBuf, response_sender: &Sender<Response>) {
    // drop every request if MusicDir has not been reset yet
    if *state == State::WaitingForReset {
        return;
    }
    match File::open(&path) {
        Ok(file) => {
            let metadata = get_track_metadata(&path);
            response_sender
                .send(Response::Track(file, Arc::from(metadata)))
                .unwrap();
            // println!("Loader: Load response sent ({path:?})");
        }
        Err(e) => {
            // files were moved, MusicDir needs a reset
            eprintln!("{e}");
            *state = State::WaitingForReset;
            response_sender.send(Response::NotFound).unwrap();
        }
    }


}

pub fn get_track_metadata(path: &Path) -> TrackMetaData {
    // get file metadata
    let mut md = extract_metadata(path).unwrap_or_default();
    if md.name == TrackMetaData::default().name {
        if let Some(name) = path.file_name() {
            if let Some(name) = name.to_str() {
                md.name = name.to_string();
            }
        }
    }

    // get duration
    let file = File::open(&path).unwrap();
    let source = Decoder::new(file).unwrap();
    let duration = source.total_duration();
    md.duration = duration;

    md
}

fn extract_metadata(path: &Path) -> Option<TrackMetaData> {
    let file = File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probe = get_probe();
    let hint = Hint::new();
    let mut probed = probe
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe format: {}", e))
        .ok()?;
    let binding = probed.metadata.get()?;
    let metadata_reader = binding.current()?;

    let mut res = TrackMetaData::default();

    // read tags
    let mut tag_found: bool = false;
    for tag in metadata_reader.tags() {
        if let Some(std_key) = tag.std_key {
            let value = tag.value.to_string();
            match std_key {
                StandardTagKey::Album => {
                    res.album = value;
                    tag_found = true;
                }
                StandardTagKey::Artist => {
                    res.artist = value;
                    tag_found = true;
                }
                StandardTagKey::TrackTitle => {
                    res.name = value;
                    tag_found = true;
                }
                _ => {}
            }
        }
    }

    if !tag_found {
        return None;
    }

    // read cover image
    if let Some(v) = metadata_reader.visuals().first() {
        res.image = get_color_image_from_visual(v);
    } else {
        res.image = get_color_image_from_track_path(path);
    }

    Some(res)
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
