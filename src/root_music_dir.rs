use crate::music_dir::MusicDir;
use std::path::PathBuf;
use std::rc::Rc;

pub struct RootMusicDir {
    parent_absolute_path: PathBuf,
    root: Rc<MusicDir>,
}

impl RootMusicDir {
    pub fn new(path: PathBuf) -> Self {
        Self {
            parent_absolute_path: path.parent().unwrap().to_path_buf(),
            root: Rc::new(MusicDir::new(path)),
        }
    }

    pub fn get_random_track_absolute_path(&self) -> Option<PathBuf> {
        Some(
            self.parent_absolute_path
                .join(self.root.get_random_track_relative_path()?),
        )
    }
}
