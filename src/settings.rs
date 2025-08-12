use crate::SETTINGS_RELATIVE_PATH;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub root_music_path: String,
    pub volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            root_music_path: "".to_string(),
            volume: 1.0,
        }
    }
}

pub fn read() -> Settings {
    match File::open(SETTINGS_RELATIVE_PATH) {
        Ok(settings_file) => serde_json::from_reader::<&File, Settings>(&settings_file).unwrap_or_else(|e| {
            eprintln!("Error in parsing settings.json: {e}");
            eprintln!("Probably due to corrupted or malformed settings file. Settings will be restored to default values.");
            let new_settings = Settings::default();
            write(&new_settings);
            new_settings
        }),
        Err(e) => {
            eprintln!("Error in reading settings.json: {e}");
            let new_settings = Settings::default();
            write(&new_settings);
            new_settings
        }
    }
}

pub fn write(data: &Settings) {
    // TODO: handle errors
    let json_string = serde_json::to_string(data).unwrap();
    let mut file = File::create(SETTINGS_RELATIVE_PATH).unwrap();
    file.write((&json_string).as_ref())
        .expect("Failed to write file");
}
