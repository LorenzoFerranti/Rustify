use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MusicDirCreationError {
    NotFound,
    NotDir,
    Empty,
    Unknown,
}

impl Display for MusicDirCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for MusicDirCreationError {}
