use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

use crate::track_metadata::TrackMetaData;

pub(crate) enum Request {
    Enqueue(File, Arc<TrackMetaData>),
    Play,
    Pause,
    JumpToFraction(f32), // [0, 1]
    Skip,
    Clear,
    SetVolume(f32), // [0, 1]
}

#[derive(Clone)]
pub(crate) enum Event {
    ProgressUpdate(Duration),
    NowPlaying,
    NowPaused,
    JumpedTo(Duration),
    NewTrackPlaying(Option<Arc<TrackMetaData>>),
    TrackFinished,
}
