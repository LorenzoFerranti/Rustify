
use std::path::{PathBuf};

pub struct MusicDir {
    pub path: PathBuf,
    pub sub_dirs: Vec<MusicDir>,
    pub tracks: Vec<PathBuf>
}

pub struct RootDir {
    root: MusicDir
}
 impl RootDir {
     pub fn new(root: PathBuf) -> Self {
         let music_dir = MusicDir {
             path: root,
             sub_dirs: vec![],
             tracks: vec![],
         };
         Self {
             root: music_dir,
         }
     }
 }
