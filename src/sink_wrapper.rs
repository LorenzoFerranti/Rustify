use std::collections::VecDeque;
use std::fs::{DirEntry, File};
use std::io::BufReader;
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::helper;

#[derive(Clone)]
pub struct Track {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
}

impl Track {
    pub fn new(name: String, artist: String, album: String, duration: Duration) -> Self {
        Self {
            name,
            artist,
            album,
            duration,
        }
    }

    pub fn default() -> Self {
        Self {
            name: "No name".to_string(),
            artist: "No artist".to_string(),
            album: "No album".to_string(),
            duration: Duration::default(),
        }
    }
}

pub struct SinkWrapper {
    output_stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
    sink: Sink,
    track_queue: VecDeque<Track>,
}

impl SinkWrapper {
    pub fn new() -> Self {
        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&output_stream_handle).unwrap();
        Self {
            output_stream,
            output_stream_handle,
            sink,
            track_queue: VecDeque::new(),
        }
    }

    pub fn clear(&mut self) {
        self.sink.clear();
        self.track_queue.clear();
    }

    pub fn append(&mut self, entry: &DirEntry) {
        let path_buf = entry.path();
        let file = BufReader::new(File::open(path_buf).unwrap());
        let source = Decoder::new(file).unwrap();

        let mut track = helper::get_track(entry).unwrap_or_else(|| Track::default());
        track.duration = source.total_duration().unwrap();

        self.delete_old_tracks();
        self.track_queue.push_back(track);
        self.sink.append(source);
    }

    pub fn skip(&mut self) {
        self.sink.skip_one();
        self.delete_old_tracks();
    }

    pub fn get_current_track(&mut self) -> Option<Track> {
        self.delete_old_tracks();
        Some(self.track_queue.front()?.clone())
    }

    pub fn get_current_track_pos(&self) -> Duration {
        self.sink.get_pos()
    }

    fn delete_old_tracks(&mut self) {
        while self.track_queue.len() > self.sink.len() {
            self.track_queue.pop_front();
        }
    }

    pub fn set_volume(&self, value: f32) {
        self.sink.set_volume(value);
    }

    pub fn set_paused(&mut self, b: bool) {
        if b {
            self.sink.pause();
        } else {
            self.sink.play();
        }
    }
}
