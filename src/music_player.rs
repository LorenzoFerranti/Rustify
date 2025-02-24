use crate::helper;

use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use std::time::Duration;

use eframe::egui::ColorImage;

use crate::root_dir::MusicDir;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

#[derive(Clone)]
pub struct TrackData {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
    pub image: Option<ColorImage>,
}

impl TrackData {
    pub fn default() -> Self {
        Self {
            name: "No name".to_string(),
            artist: "No artist".to_string(),
            album: "No album".to_string(),
            duration: Duration::default(),
            image: None,
        }
    }
}

pub struct MusicPlayer {
    output_stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
    sink: Sink,
    track_queue: VecDeque<TrackData>,
    playlist: Option<Rc<MusicDir>>, // TODO: change to Playlist struct
}

impl MusicPlayer {
    pub fn new() -> Self {
        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&output_stream_handle).unwrap();
        Self {
            output_stream,
            output_stream_handle,
            sink,
            track_queue: VecDeque::new(),
            playlist: None,
        }
    }

    pub fn clear(&mut self) {
        self.sink.clear();
        self.track_queue.clear();
    }

    fn append_random(&mut self) {
        if let Some(md_pt) = &self.playlist {
            if let Some(path) = md_pt.get_random_track_path() {
                let file = BufReader::new(File::open(&path).unwrap());
                let source = Decoder::new(file).unwrap();

                let mut track_data =
                    helper::get_track_data(path).unwrap_or_else(TrackData::default);
                track_data.duration = source.total_duration().unwrap_or_else(|| {
                    println!("No duration!!");
                    Duration::from_secs(1)
                });

                self.delete_old_tracks();
                self.track_queue.push_back(track_data);
                self.sink.append(source);
            }
        }
    }

    pub fn skip(&mut self) {
        self.sink.skip_one();
        self.delete_old_tracks_and_refill_queue();
    }

    pub fn get_current_track(&mut self) -> Option<TrackData> {
        self.delete_old_tracks_and_refill_queue();
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

    // TODO: this gets called at every redraw. BAD! create a thread
    fn delete_old_tracks_and_refill_queue(&mut self) {
        self.delete_old_tracks();
        if self.playlist.is_some() {
            while self.sink.len() < 3 {
                self.append_random();
            }
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

    pub fn get_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn jump(&mut self, point: f32) -> Option<()> {
        let d = self.get_current_track()?.duration.mul_f32(point);
        self.sink.try_seek(d).ok()
    }

    // TODO: change to Playlist struct
    pub fn set_playlist(&mut self, md_ptr: Rc<MusicDir>) {
        self.clear();
        self.playlist = Some(md_ptr);
        self.set_paused(false);
    }
}
