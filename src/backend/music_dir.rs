use std::ffi::OsString;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use crate::music_dir_creation_error::MusicDirCreationError;
use rand::random;

pub(crate) struct MusicDir {
    root_path: Option<PathBuf>,
    music_dir: _MusicDir,
}

impl MusicDir {
    pub(crate) fn new(root_path: &Path) -> Result<Self, MusicDirCreationError> {
        let music_dir = _MusicDir::new(root_path)?;
        let new_root_path = root_path.parent().and_then(|p| Some(p.to_path_buf()));
        Ok(Self {
            root_path: new_root_path,
            music_dir,
        })
    }

    pub(crate) fn get_random_track_path(&self) -> Option<PathBuf> {
        let relative_path = self.music_dir.get_random_track_path()?;
        match &self.root_path {
            None => Some(relative_path),
            Some(p) => Some(p.join(relative_path)),
        }
    }
}

struct _MusicDir {
    name: OsString,
    sub_dirs: Vec<_MusicDir>,
    track_names: Vec<OsString>,
}

impl _MusicDir {
    fn new(path: &Path) -> Result<Self, MusicDirCreationError> {
        println!("Creating {}", path.display());
        if !path.exists() {
            println!("DOESNT EXIST");
            return Err(MusicDirCreationError::NotFound);
        }
        if !path.is_dir() {
            println!("NOT DIR");
            return Err(MusicDirCreationError::NotDir);
        }
        let name = match path.file_name() {
            None => return Err(MusicDirCreationError::Unknown),
            Some(name) => name.to_os_string(),
        };
        let tracks = get_all_track_names(&path);
        let sub_dirs = get_sub_dirs(&path);
        if tracks.is_none() && sub_dirs.is_none() {
            println!("EMPTY");
            Err(MusicDirCreationError::Empty)
        } else {
            println!("{} created!!!!!", path.display());
            Ok(Self {
                name,
                sub_dirs: sub_dirs.unwrap_or_default(),
                track_names: tracks.unwrap_or_default(),
            })
        }
    }

    fn has_tracks(&self) -> bool {
        !self.track_names.is_empty()
    }

    fn has_sub_dirs(&self) -> bool {
        !self.sub_dirs.is_empty()
    }

    fn get_random_track_path(&self) -> Option<PathBuf> {
        if self.has_tracks() {
            let n = get_random_index(&self.track_names);
            let res = Some(Path::new(&self.name).join(PathBuf::from(&self.track_names[n])));
            return res;
        }
        if self.has_sub_dirs() {
            let n = get_random_index(&self.sub_dirs);
            let sub_path = self.sub_dirs[n].get_random_track_path()?;
            let res = Some(PathBuf::from(&self.name).join(sub_path));
            return res;
        }
        None
    }
}

fn get_all_track_names(path: &Path) -> Option<Vec<OsString>> {
    const VALID_EXTENSIONS: [&str; 1] = ["mp3"];
    let mut res = vec![];
    let dir_iter = read_dir(path).ok()?;

    for entry in dir_iter.flatten() {
        let entry_path = entry.path();

        if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
            if VALID_EXTENSIONS.contains(&ext) {
                if let Some(name) = entry_path.file_name() {
                    res.push(name.to_os_string())
                }
            }
        }
    }

    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}

fn get_sub_dirs(path: &Path) -> Option<Vec<_MusicDir>> {
    let mut res = vec![];
    match read_dir(path) {
        Ok(dir_iter) => {
            for entry in dir_iter.flatten() {
                let entry_path = entry.path();
                match _MusicDir::new(&entry_path) {
                    Ok(music_dir) => {
                        res.push(music_dir);
                    }
                    Err(_) => {}
                }
            }
            if res.is_empty() {
                None
            } else {
                Some(res)
            }
        }
        Err(e) => {
            eprintln!("Error in reading dir {}: {e}", path.display());
            None
        }
    }
}

fn get_random_index<T>(v: &[T]) -> usize {
    random::<usize>() % v.len()
}