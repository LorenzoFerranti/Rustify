use std::process::exit;
use std::thread;

use crossbeam_channel::unbounded;
use eframe::egui::ViewportBuilder;

use crate::messages::Event;

mod backend;
mod frontend;
mod image_utils;
mod messages;
mod music_dir_creation_error;
mod settings;
mod track_metadata;

pub const SETTINGS_RELATIVE_PATH: &str = "settings.json";

fn main() -> eframe::Result {
    // create channels
    let (req_sender, req_receiver) = unbounded::<messages::Request>();
    let (event_sender, event_receiver) = unbounded::<messages::Event>();

    // spawn backend thread
    thread::spawn(move || backend::run(req_receiver, event_sender));

    // wait for initial settings message
    let settings = match event_receiver.recv() {
        Ok(event) => {
            if let Event::NewSettings(s) = event {
                s
            } else {
                eprintln!("Error: first message is not loaded settings!");
                eprintln!("First message was: {event:?}");
                exit(1)
            }
        }
        Err(e) => {
            eprintln!("Error: {e:?}");
            exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size((600.0, 600.0))
            .with_min_inner_size((400.0, 300.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Rustify",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(frontend::App::new(
                cc,
                settings,
                req_sender,
                event_receiver,
            )))
        }),
    )
}
