use std::cmp::Ordering;
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
        let new_root_path = root_path.parent().map(Path::to_path_buf);
        Ok(Self {
            root_path: new_root_path,
            music_dir,
        })
    }

    pub(crate) fn get_next_track_path(&mut self) -> PathBuf {
        let relative_path = self.music_dir.get_next_track_path();
        match &self.root_path {
            None => relative_path,
            Some(p) => p.join(relative_path),
        }
    }

    pub(crate) fn print_tree(&self) {
        self.music_dir.print_tree(0);
    }
}

struct _MusicDir {
    name: OsString,
    total_sub_tracks_played: u32,
    total_sub_tracks: u32,
    sub_dirs: Vec<_MusicDir>,
    total_local_tracks_played: u32,
    local_tracks: Vec<(OsString, u32)>,
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
        let tracks = get_all_tracks(path);
        let sub_dirs = get_sub_dirs(path);
        if tracks.is_none() && sub_dirs.is_none() {
            Err(MusicDirCreationError::Empty)
        } else {
            let mut total_sub_tracks = 0;

            // let mut total_sub_tracks = match tracks.as_ref() {
            //     None => 0,
            //     Some(v) => v.len() as u32,
            // };
            if let Some(dirs) = sub_dirs.as_ref() {
                for dir in dirs {
                    total_sub_tracks += dir.get_total_tracks();
                }
            }
            Ok(Self {
                name,
                total_sub_tracks_played: 0,
                total_sub_tracks,
                sub_dirs: sub_dirs.unwrap_or_default(),
                total_local_tracks_played: 0,
                local_tracks: tracks.unwrap_or_default(),
            })
        }
    }

    fn get_next_track_path(&mut self) -> PathBuf {
        let mut least_played_factor: Option<f32> = self.get_local_played_factor();
        let mut least_played_dirs: Vec<&mut _MusicDir> = Vec::new();

        for dir in self.sub_dirs.iter_mut() {
            match least_played_factor {
                None => {
                    least_played_factor = Some(dir.get_played_factor());
                    least_played_dirs.push(dir);
                }
                Some(lpf) => match dir.get_played_factor().partial_cmp(&lpf) {
                    None => unreachable!(),
                    Some(ord) => match ord {
                        Ordering::Less => {
                            least_played_factor = Some(dir.get_played_factor());
                            least_played_dirs.clear();
                            least_played_dirs.push(dir);
                        }
                        Ordering::Equal => {
                            least_played_dirs.push(dir);
                        }
                        Ordering::Greater => {}
                    },
                },
            }
        }

        if least_played_dirs.is_empty() {
            match least_played_factor {
                None => unreachable!(), // this means self has no local tracks and no sub dirs
                Some(_) => {
                    self.total_local_tracks_played += 1;
                    // TODO: dont clone
                    let track_name = self.get_least_played_local_track().clone();
                    Path::new(&self.name).join(PathBuf::from(track_name))
                }
            }
        } else {
            self.total_sub_tracks_played += 1;
            let index = get_random_index(&least_played_dirs);
            let sub_path = least_played_dirs[index].get_next_track_path();
            PathBuf::from(&self.name).join(sub_path)
        }
    }

    fn get_total_tracks(&self) -> u32 {
        self.total_sub_tracks + (self.local_tracks.len() as u32)
    }

    fn get_played_factor(&self) -> f32 {
        (self.total_local_tracks_played + self.total_sub_tracks_played) as f32
            / (self.local_tracks.len() as f32 + self.total_sub_tracks as f32)
    }

    fn get_sub_played_factor(&self) -> Option<f32> {
        if self.total_sub_tracks > 0 {
            Some((self.total_sub_tracks_played as f32) / (self.total_sub_tracks as f32))
        } else {
            None
        }
    }

    fn get_local_played_factor(&self) -> Option<f32> {
        if self.local_tracks.is_empty() {
            None
        } else {
            Some((self.total_local_tracks_played as f32) / (self.local_tracks.len() as f32))
        }
    }

    fn get_least_played_local_track(&mut self) -> &OsString {
        let mut least_played_factor: Option<u32> = None;
        let mut least_played_tracks_ids: Vec<usize> = Vec::new();

        for (track_index, (_, played_factor)) in self.local_tracks.iter().enumerate() {
            match least_played_factor {
                None => {
                    least_played_factor = Some(*played_factor);
                    least_played_tracks_ids.push(track_index);
                }
                Some(lpf) => match (*played_factor).cmp(&lpf) {
                    Ordering::Less => {
                        least_played_factor = Some(*played_factor);
                        least_played_tracks_ids.clear();
                        least_played_tracks_ids.push(track_index);
                    }
                    Ordering::Equal => {
                        least_played_tracks_ids.push(track_index);
                    }
                    Ordering::Greater => {}
                },
            }
        }
        let random_index = get_random_index(&least_played_tracks_ids);
        let track_index = least_played_tracks_ids[random_index];
        self.local_tracks[track_index].1 += 1;
        &self.local_tracks[track_index].0
    }

    fn print_tree(&self, indent: u32) {
        for _ in 0..indent {
            print!("    ");
        }
        println!(
            "{} - lt:{}, st:{}, pf:{}, lpf:{:?}, spf:{:?}",
            self.name.to_string_lossy(),
            self.local_tracks.len(),
            self.total_sub_tracks,
            self.get_played_factor(),
            self.get_local_played_factor(),
            self.get_sub_played_factor(),
        );
        for (name, played) in &self.local_tracks {
            for _ in 0..=indent {
                print!("    ");
            }
            println!("{name:?} - {played}");
        }
        for dir in &self.sub_dirs {
            dir.print_tree(indent + 1);
        }
    }
}

fn get_all_tracks(path: &Path) -> Option<Vec<(OsString, u32)>> {
    const VALID_EXTENSIONS: [&str; 1] = ["mp3"];
    let mut res = vec![];
    let dir_iter = read_dir(path).ok()?;

    for entry in dir_iter.flatten() {
        let entry_path = entry.path();

        if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
            if VALID_EXTENSIONS.contains(&ext) {
                if let Some(name) = entry_path.file_name() {
                    res.push((name.to_os_string(), 0));
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
                if let Ok(music_dir) = _MusicDir::new(&entry_path) {
                    res.push(music_dir);
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

// TODO: check if length > 0
fn get_random_index<T>(v: &[T]) -> usize {
    random::<usize>() % v.len()
}
