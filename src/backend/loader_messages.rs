use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use crate::track_metadata::TrackMetaData;

#[derive(Clone)]
pub(crate) enum Request {
    Track(PathBuf),
}

pub(crate) enum Response {
    Track(File, Arc<TrackMetaData>),
    NotFound(PathBuf),
}
