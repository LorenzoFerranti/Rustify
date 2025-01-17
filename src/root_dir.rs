use std::fs::{read_dir};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use rand::random;

pub struct MusicDir {
    pub path: PathBuf,
    pub sub_dirs: Vec<Rc<MusicDir>>,
    pub track_paths: Vec<PathBuf>
}

impl MusicDir {
    pub fn new(path: PathBuf) -> Self {
        println!("music dir: {:?}", path);
        let tracks = get_mp3s(&path).unwrap_or(vec![]);
        let sub_dirs = get_sub_dirs(&path).unwrap_or(vec![]);
        Self {
            path,
            sub_dirs,
            track_paths: tracks,
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

pub struct RootDir {
    pub root: Rc<MusicDir>
}
impl RootDir {
    pub fn new(root: PathBuf) -> Self {
        let root = Rc::new(MusicDir::new(root));
        Self {
            root
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

fn get_sub_dirs(path: &Path) -> Option<Vec<Rc<MusicDir>>> {
    let mut res = vec![];
    let read_dir = read_dir(path).ok()?;

    for entry in read_dir.flatten() {
        let path_buf = entry.path();
        if path_buf.is_dir() {
            let music_dir_ptr = Rc::new(MusicDir::new(path_buf));
            if !music_dir_ptr.is_empty() {
                res.push(music_dir_ptr);
            }
        }
    }

    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}

fn get_random_index<T>(v: &[T]) -> usize {
    random::<usize>() % v.len()
}