mod sink_wrapper;

use eframe::egui::{Context, CentralPanel, ViewportBuilder, TextEdit, TextStyle, Key, ProgressBar, Slider, Checkbox};
use eframe::{Frame};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use std::fs::{read_dir, File};
use std::io::BufReader;
use std::path::PathBuf;
use rand::random;

use crate::sink_wrapper::SinkWrapper;

const MY_LOCAL_PATH: &str = "C:\\Users\\loren\\Desktop\\OSTs";

struct RustifyApp {
    path: String,
    sink: SinkWrapper,
    volume_input: f32,
    paused_input: bool,
}

impl RustifyApp {
    pub fn new() -> Self {
        Self {
            path: MY_LOCAL_PATH.to_string(),
            sink: SinkWrapper::new(),
            volume_input: 1.0,
            paused_input: false,
        }
    }

    pub fn play_random_from_path(&mut self) {
        self.sink.clear();
        let mut current_path = self.path.clone();
        loop {
            if let Some(v) = Self::get_mp3_pathbufs(&current_path) {
                self.append_music(&v[get_random_index(&v)]);
                self.sink.play();
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

    fn append_music(&mut self, path_buf: &PathBuf) {
        println!("appending {:?}", path_buf.to_str());
        let file = BufReader::new(File::open(path_buf).unwrap());
        let source = Decoder::new(file).unwrap();
        self.sink.append(source);
    }


    fn get_mp3_pathbufs(path: &str) -> Option<Vec<PathBuf>> {
        let mut res = vec![];
        let dir_iter = read_dir(path).ok()?;

        for entry in dir_iter.flatten() {
            let path_buf = entry.path();
            if let Some(ext) = path_buf.extension() {
                if ext == "mp3" {
                    res.push(path_buf);
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
        CentralPanel::default().show(ctx, |ui| {
            if ui.button("Play random track").clicked() {
                self.play_random_from_path();
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
        Box::new(|_| Ok(Box::new(RustifyApp::new()))),
    )
}

fn get_random_index<T>(v: &[T]) -> usize {
    random::<usize>() % v.len()
}
