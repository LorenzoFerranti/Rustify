use std::fs::File;
use std::io::Write;
use std::thread;
use std::path::PathBuf;
use std::time::Duration;

use eframe::egui::{
    Align, Button, CentralPanel, Color32, Context, Image, Layout, RichText, Slider, TextEdit,
    TextStyle, TextureOptions, TopBottomPanel, Ui,
};
use eframe::{CreationContext, Frame};

use serde::{Serialize, Deserialize};

use crate::helper;
use crate::music_player::MusicPlayer;
use crate::root_music_dir::RootMusicDir;

const DATA_RELATIVE_PATH: &str = "data.json";

#[derive(Debug, Serialize, Deserialize)]
struct RustifyOptions {
    root_path: String,
    volume: f32,
}

impl Default for RustifyOptions {
    fn default() -> Self {
        Self {
            root_path: "".to_string(),
            volume: 1.0,
        }
    }
}

pub struct RustifyApp {
    options: RustifyOptions,
    music_player: MusicPlayer,
    duration_slider: f32,
}

impl RustifyApp {
    pub fn new(cc: &CreationContext) -> Self {
        // repaint thread
        let ctx_clone = cc.egui_ctx.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(500));
            ctx_clone.request_repaint();
        });

        // read data
        let options = Self::read_options().unwrap_or_else(|| {
            let new_data = RustifyOptions::default();
            Self::write_options(&new_data);
            new_data
        });

        Self {
            options,
            music_player: MusicPlayer::new(),
            duration_slider: 0.0,
        }
    }

    fn read_options() -> Option<RustifyOptions> {
        let file = File::open(DATA_RELATIVE_PATH).ok()?;
        let data : RustifyOptions = serde_json::from_reader(&file).ok()?;
        Some(data)
    }

    fn write_options(data: &RustifyOptions) {
        let json_string = serde_json::to_string(data).unwrap();
        let mut file = File::create(DATA_RELATIVE_PATH).unwrap();
        file.write((&json_string).as_ref()).expect("Failed to write file");
    }

    fn save_options(&self) {
        Self::write_options(&self.options);
    }

    fn spawn_duration_slider(&mut self, ui: &mut Ui) {
        ui.spacing_mut().slider_width = ui.available_width();
        let response = ui.add(
            Slider::new(&mut self.duration_slider, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );
        if response.drag_stopped() {
            self.music_player.jump(self.duration_slider);
        }
    }

    pub fn spawn_pause_button(&mut self, ui: &mut Ui) {
        let text = if self.music_player.get_paused() {
            "▶"
        } else {
            "⏸"
        };
        let response = ui.add_sized(
            [40.0, 40.0],
            Button::new(RichText::new(text).size(20.0)).rounding(7.0),
        );
        if response.clicked() {
            self.music_player
                .set_paused(!self.music_player.get_paused())
        }
    }

    pub fn spawn_skip_button(&mut self, ui: &mut Ui) {
        let text = "⏭";
        let response = ui.add_sized(
            [40.0, 40.0],
            Button::new(RichText::new(text).size(20.0)).rounding(7.0),
        );
        if response.clicked() {
            self.music_player.skip();
        }
    }
}

impl eframe::App for RustifyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let mut enable_duration_bar = true;

                if let Some(track) = self.music_player.get_current_track() {
                    // get pos
                    let pos = self.music_player.get_current_track_pos();
                    self.duration_slider = pos.as_secs_f32().floor() / track.duration.as_secs_f32();
                    // name, artist and album
                    ui.add_space(5.0);
                    ui.horizontal_wrapped(|ui| {
                        let text = RichText::new(track.name).color(Color32::WHITE);
                        ui.heading(text);
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{} - {}", track.artist, track.album));
                    });
                } else {
                    enable_duration_bar = false;
                    self.duration_slider = 0.0;
                }

                ui.add_space(5.0);

                // slider
                ui.horizontal(|ui| {
                    ui.label(helper::formatted_duration(
                        &self.music_player.get_current_track_pos(),
                    ));
                    // layout needed for correct expansion of the slider
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(track) = self.music_player.get_current_track() {
                            ui.label(helper::formatted_duration(&track.duration));
                        } else {
                            ui.label("00:00");
                        }
                        ui.add_enabled_ui(enable_duration_bar, |ui| {
                            self.spawn_duration_slider(ui);
                        });
                    });
                });
                ui.columns(3, |cols| {
                    cols[0].vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label("Volume");
                            let response = ui.add(
                                Slider::new(&mut self.options.volume, 0.0..=1.0).show_value(false),
                            );
                            if response.changed() {
                                self.music_player.set_volume(self.options.volume);
                            }
                            if response.drag_stopped() {
                                self.save_options();
                            }
                        });
                    });
                    cols[1].vertical_centered(|ui| self.spawn_pause_button(ui));
                    cols[2].vertical_centered(|ui| self.spawn_skip_button(ui));
                });

                ui.add_space(5.0);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.label("Root path");
            ui.add(
                TextEdit::singleline(&mut self.options.root_path)
                    .desired_width(f32::INFINITY)
                    .font(TextStyle::Monospace),
            );
            if ui.button("Play root").clicked() {
                self.save_options();
                self.music_player
                    .set_playlist(RootMusicDir::new(PathBuf::from(&self.options.root_path)));
            }
            ui.add_space(15.0);

            if let Some(track) = self.music_player.get_current_track() {
                if let Some(color_image) = track.image {
                    let texture =
                        ctx.load_texture("my_texture", color_image, TextureOptions::default());
                    ui.add(Image::new(&texture).max_width(200.0).rounding(10.0));
                } else {
                    ui.label("NO image");
                }
            }
        });
    }
}
