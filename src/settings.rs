use crate::SETTINGS_RELATIVE_PATH;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::process;

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
            eprintln!("Error in parsing {SETTINGS_RELATIVE_PATH}: {e}");
            eprintln!("Probably due to corrupted or malformed settings file. Settings will be restored to default values.");
            let new_settings = Settings::default();
            write(&new_settings);
            new_settings
        }),
        Err(e) => {
            eprintln!("Error in reading {SETTINGS_RELATIVE_PATH}: {e}");
            let new_settings = Settings::default();
            write(&new_settings);
            new_settings
        }
    }
}

pub fn write(data: &Settings) {
    let json_string = serde_json::to_string(data).unwrap_or_else(|e| {
        eprintln!("Failed to serialize settings: {e}");
        process::exit(1);
    });

    let mut file = File::create(SETTINGS_RELATIVE_PATH).unwrap_or_else(|e| {
        eprintln!("Failed to create file '{}': {e}", SETTINGS_RELATIVE_PATH);
        process::exit(1);
    });

    file.write_all(json_string.as_ref()).unwrap_or_else(|e| {
        eprintln!("Failed to write to file '{}': {e}", SETTINGS_RELATIVE_PATH);
        process::exit(1);
    });
}
