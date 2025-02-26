use rand::random;
use std::cell::RefCell;
use std::ffi::OsString;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct MusicDir {
    pub name: OsString,
    pub sub_dirs: Vec<Rc<MusicDir>>,
    pub track_names: Vec<OsString>,
}

impl MusicDir {
    pub fn new(path: PathBuf) -> Self {
        //println!("music dir: {:?}", path);
        let tracks = get_mp3s(&path).unwrap_or_default();
        let sub_dirs = get_sub_dirs(&path).unwrap_or_default();
        Self {
            name: path.file_name().unwrap().to_os_string(),
            sub_dirs,
            track_names: tracks,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.track_names.is_empty() && self.sub_dirs.is_empty()
    }
    pub fn has_tracks(&self) -> bool {
        !self.track_names.is_empty()
    }

    pub fn has_sub_dirs(&self) -> bool {
        !self.sub_dirs.is_empty()
    }

    pub fn get_random_track_relative_path(&self) -> Option<PathBuf> {
        if self.has_tracks() {
            let n = get_random_index(&self.track_names);
            return Some(PathBuf::from(&self.name).join(&self.track_names[n]));
        }
        if self.has_sub_dirs() {
            let n = get_random_index(&self.sub_dirs);
            let relative_path = self.sub_dirs[n].get_random_track_relative_path()?;
            return Some(PathBuf::from(&self.name).join(relative_path));
        }
        None
    }
}

fn get_mp3s(path: &Path) -> Option<Vec<OsString>> {
    let mut res = vec![];
    let read_dir = read_dir(path).ok()?;

    for entry in read_dir.flatten() {
        let path_buf = entry.path();
        if let Some(ext) = path_buf.extension() {
            if ext == "mp3" {
                res.push(path_buf.file_name().unwrap().to_os_string());
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

    for entry in read_dir(path).ok()?.flatten() {
        let path_buf = entry.path();
        if path_buf.is_dir() {
            let mut music_dir_ptr = Rc::new(MusicDir::new(path_buf));
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

impl Drop for MusicDir {
    fn drop(&mut self) {
        println!("Dropping {:?}", self.name);
    }
}

fn get_random_index<T>(v: &[T]) -> usize {
    random::<usize>() % v.len()
}
