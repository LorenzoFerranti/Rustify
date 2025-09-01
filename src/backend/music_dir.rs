use rand::random;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct MusicDir {
    path: PathBuf,
    sub_dirs: Vec<Rc<MusicDir>>,
    track_paths: Vec<PathBuf>,
}

#[derive(Debug)]
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

impl MusicDir {
    pub fn new(path: PathBuf) -> Result<Self, MusicDirCreationError> {
        if !path.exists() {
            return Err(MusicDirCreationError::NotFound);
        }
        if !path.is_dir() {
            return Err(MusicDirCreationError::NotDir);
        }
        let tracks = get_all_mp3s(&path);
        let sub_dirs = get_sub_dirs(&path);
        if tracks.is_none() && sub_dirs.is_err() {
            Err(MusicDirCreationError::Empty)
        } else {
            Ok(Self {
                path,
                sub_dirs: sub_dirs.unwrap_or_default(),
                track_paths: tracks.unwrap_or_default(),
            })
        }
    }

    pub fn is_empty(&self) -> bool {
        self.track_paths.is_empty() && self.sub_dirs.is_empty()
    }
    pub fn has_tracks(&self) -> bool {
        !self.track_paths.is_empty()
    }

    pub fn has_sub_dirs(&self) -> bool {
        !self.sub_dirs.is_empty()
    }

    pub fn get_random_track_path(&self) -> Option<PathBuf> {
        if self.has_tracks() {
            let n = get_random_index(&self.track_paths);
            return Some(self.track_paths[n].clone());
        }
        if self.has_sub_dirs() {
            let n = get_random_index(&self.sub_dirs);
            return self.sub_dirs[n].get_random_track_path();
        }
        None
    }
}

fn get_all_mp3s(path: &Path) -> Option<Vec<PathBuf>> {
    let mut res = vec![];
    let read_dir = read_dir(path).ok()?;

    for entry in read_dir.flatten() {
        let path_buf = entry.path();
        if let Some(ext) = path_buf.extension() {
            if ext == "mp3" {
                res.push(path_buf);
            }
        }
    }

    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}

fn get_sub_dirs(path: &Path) -> Result<Vec<Rc<MusicDir>>, MusicDirCreationError> {
    let mut res = vec![];
    match read_dir(path) {
        Ok(dir_iter) => {
            for entry in dir_iter.flatten() {
                let path_buf = entry.path();
                match MusicDir::new(path_buf) {
                    Ok(music_dir) => {
                        res.push(Rc::new(music_dir));
                    }
                    Err(e) => match e {
                        MusicDirCreationError::NotFound => unreachable!(),
                        MusicDirCreationError::NotDir => {}
                        MusicDirCreationError::Empty => {}
                        MusicDirCreationError::Unknown => {
                            return Err(MusicDirCreationError::Unknown);
                        }
                    },
                }
            }
            if res.is_empty() {
                Err(MusicDirCreationError::Empty)
            } else {
                Ok(res)
            }
        }
        Err(e) => {
            eprintln!("Error in reading dir {}: {e}", path.display());
            Err(MusicDirCreationError::Unknown)
        }
    }
}

fn get_random_index<T>(v: &[T]) -> usize {
    random::<usize>() % v.len()
}
