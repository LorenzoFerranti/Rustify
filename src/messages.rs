use crate::settings::Settings;
use crate::track_metadata::TrackMetaData;
use eframe::egui::Context;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub enum Request {
    ChangeRoot(PathBuf),
    Play,
    Pause,
    JumpToFraction(f32), // [0, 1]
    Skip,
    SetVolume(f32), // [0, 1]
    ProvideContext(Context),
}

#[derive(Debug)]
pub enum Event {
    NewTrackPlaying(Option<Arc<TrackMetaData>>),
    NowPlaying,
    NowPaused,
    ProgressUpdate(Duration), // [0, 1], always forward
    JumpedTo(Duration),       // [0, 1]
    NewSettings(Settings),
}
