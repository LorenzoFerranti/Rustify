use std::fs::{read_dir};
use std::path::{Path, PathBuf};

pub struct MusicDir {
    pub path: PathBuf,
    pub sub_dirs: Vec<MusicDir>,
    pub tracks: Vec<PathBuf>
}

impl MusicDir {
    pub fn new(path: PathBuf) -> Self {
        println!("music dir: {:?}", path);
        let tracks = get_mp3s(&path).unwrap_or(vec![]);
        let sub_dirs = get_sub_dirs(&path).unwrap_or(vec![]);
        Self {
            path,
            sub_dirs,
            tracks,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty() && self.sub_dirs.is_empty()
    }
}

pub struct RootDir {
    root: MusicDir
}
impl RootDir {
    pub fn new(root: PathBuf) -> Self {
        let music_dir = MusicDir::new(root);
        Self {
            root: music_dir,
        }
    }
}

fn get_mp3s(path: &Path) -> Option<Vec<PathBuf>> {
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

fn get_sub_dirs(path: &Path) -> Option<Vec<MusicDir>> {
    let mut res = vec![];
    let read_dir = read_dir(path).ok()?;

    for entry in read_dir.flatten() {
        let path_buf = entry.path();
        if path_buf.is_dir() {
            let music_dir = MusicDir::new(path_buf);
            if !music_dir.is_empty() {
                res.push(music_dir);
            }
        }
    }

    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}