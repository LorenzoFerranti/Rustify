use std::fs::File;
use std::io::Write;
use crate::ui::app::{RustifyApp, RustifyOptions};

impl RustifyApp {
    pub(crate) fn save_options(&self) {
        write_options(&self.options);
    }

    pub(crate) fn get_options() -> RustifyOptions {
        read_options().unwrap_or_else(|| {
            let new_data = RustifyOptions::default();
            write_options(&new_data);
            new_data
        })
    }
}

fn read_options() -> Option<RustifyOptions> {
    let file = File::open(crate::ui::app::DATA_RELATIVE_PATH).ok()?;
    let data : RustifyOptions = serde_json::from_reader(&file).ok()?;
    Some(data)
}

fn write_options(data: &RustifyOptions) {
    let json_string = serde_json::to_string(data).unwrap();
    let mut file = File::create(crate::ui::app::DATA_RELATIVE_PATH).unwrap();
    file.write((&json_string).as_ref()).expect("Failed to write file");
}