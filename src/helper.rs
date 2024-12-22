use crate::sink_wrapper::Track;

use std::fs::{DirEntry, File};

use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

pub fn get_track(entry: &DirEntry) -> Option<Track> {
    // Open the file
    let path_buf = entry.path();
    let file = File::open(path_buf).unwrap();
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Get the default codec and format registry
    let probe = get_probe();
    let hint = Hint::new();

    // Probe the file for format information
    let mut probed = probe
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe format: {}", e))
        .ok()?;

    // Get metadata
    if let Some(metadata) = probed.metadata.get() {
        let current_metadata = metadata.current().unwrap();
        let mut track = Track::default();

        for tag in current_metadata.tags() {
            if let Some(std_key) = tag.std_key {
                let value = tag.value.to_string();
                match std_key {
                    StandardTagKey::Album => track.album = value,
                    StandardTagKey::Artist => track.artist = value,
                    StandardTagKey::TrackTitle => track.name = value,
                    x => {
                        println!("matched {:?} = {:?}", x, tag.value)
                    }
                }
            }
        }

        return Some(track);
    }
    None
}
