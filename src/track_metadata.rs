use eframe::egui::ColorImage;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct TrackMetaData {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: Option<Duration>,
    pub image: Option<ColorImage>,
}

impl TrackMetaData {
    pub fn default() -> Self {
        Self {
            name: "No name".to_string(),
            artist: "No artist".to_string(),
            album: "No album".to_string(),
            duration: None,
            image: None,
        }
    }
}
