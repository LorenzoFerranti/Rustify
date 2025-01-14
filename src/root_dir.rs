use std::fs::{read_dir, DirEntry};
use std::path::{Path, PathBuf};

pub struct MusicDir {
    pub path: PathBuf,
    pub sub_dirs: Vec<MusicDir>,
    pub tracks: Vec<PathBuf>
}

pub struct RootDir {
    root: MusicDir
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

fn get_sub_dirs(path: &Path) -> Option<Vec<PathBuf>> {
    let mut res = vec![];
    let read_dir = read_dir(path).ok()?;

    for entry in read_dir.flatten() {
        let path_buf = entry.path();
        if path_buf.is_dir() {
            res.push(path_buf);
        }
    }

    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}