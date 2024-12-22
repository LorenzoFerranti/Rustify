mod helper;
mod sink_wrapper;

use rand::random;
use std::fs::{read_dir, DirEntry, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use eframe::egui::{
    CentralPanel, Checkbox, Context, ProgressBar, Rounding, Slider, TextEdit, TextStyle,
    ViewportBuilder,
};
use eframe::{CreationContext, Frame};

use rodio::{Decoder, Source};

use crate::sink_wrapper::SinkWrapper;

const MY_LOCAL_PATH: &str = "C:\\Users\\loren\\Desktop\\OSTs\\Short";

struct RustifyApp {
    path: String,
    sink: SinkWrapper,
    volume_input: f32,
    paused_input: bool,
}

impl RustifyApp {
    pub fn new(cc: &CreationContext) -> Self {
        let ctx_clone = cc.egui_ctx.clone();
        std::thread::spawn(move || loop {
            sleep(Duration::from_millis(500));
            ctx_clone.request_repaint();
        });
        Self {
            path: MY_LOCAL_PATH.to_string(),
            sink: SinkWrapper::new(),
            volume_input: 1.0,
            paused_input: false,
        }
    }

    pub fn append_random_from_path(&mut self) {
        let mut current_path = self.path.clone();
        loop {
            if let Some(v) = Self::get_mp3_dir_entries(&current_path) {
                self.sink.append(&v[get_random_index(&v)]);
                return;
            }
            // no .mp3 files
            if let Some(v) = Self::get_dir_paths(&current_path) {
                current_path = v[get_random_index(&v)].clone();
            } else {
                println!("Error: no mp3 and no directories");
                return;
            }
        }
    }

    fn get_mp3_dir_entries(path: &str) -> Option<Vec<DirEntry>> {
        let mut res = vec![];
        let dir_iter = read_dir(path).ok()?;

        for entry in dir_iter.flatten() {
            let path_buf = entry.path();
            if let Some(ext) = path_buf.extension() {
                if ext == "mp3" {
                    res.push(entry);
                }
            }
        }

        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    fn get_dir_paths(path: &str) -> Option<Vec<String>> {
        let mut res = vec![];
        let dir_iter = read_dir(path).ok()?;

        for entry in dir_iter.flatten() {
            let path_buf = entry.path();
            if path_buf.is_dir() {
                if let Some(s) = path_buf.to_str() {
                    res.push(s.to_string());
                }
            }
        }

        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }
}

impl eframe::App for RustifyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        //println!("Update!!{:?}", Instant::now());
        CentralPanel::default().show(ctx, |ui| {
            if ui.button("Add random track to queue").clicked() {
                self.append_random_from_path();
            }
            if ui.button("Skip track").clicked() {
                self.sink.skip();
            }
            ui.add_space(15.0);

            ui.label("Root path");
            ui.add(
                TextEdit::singleline(&mut self.path)
                    .desired_width(f32::INFINITY)
                    .font(TextStyle::Monospace),
            );
            ui.add_space(15.0);

            ui.label("Volume");
            let response = ui.add(Slider::new(&mut self.volume_input, 0.0..=1.0));
            if response.changed() {
                self.sink.set_volume(self.volume_input);
            }
            let response = ui.add(Checkbox::new(&mut self.paused_input, "Paused"));
            if response.changed() {
                self.sink.set_paused(self.paused_input);
            }

            let mut progress = 0.0;
            if let Some(track) = self.sink.get_current_track() {
                let pos = self.sink.get_current_track_pos();
                progress = pos.as_secs_f32().floor() / track.duration.as_secs_f32();
                ui.label(format!("Track name: {}", track.name));
                ui.label(format!("Album: {}", track.album));
                ui.label(format!("Artist(s): {}", track.artist));
                ui.label(format!("{:?}", pos));
                ui.label(format!("{:?}", track.duration));
            }

            ui.add(ProgressBar::new(progress).rounding(Rounding::ZERO));
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size((600.0, 600.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Rustify",
        native_options,
        Box::new(|cc| Ok(Box::new(RustifyApp::new(cc)))),
    )
}

fn get_random_index<T>(v: &[T]) -> usize {
    random::<usize>() % v.len()
}
