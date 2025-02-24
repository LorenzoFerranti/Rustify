use crate::helper::formatted_duration;
use crate::music_player::MusicPlayer;

use std::path::PathBuf;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::root_dir::RootDir;
use eframe::egui::{
    Align, Button, CentralPanel, Color32, Context, Image, Layout, RichText, Slider, TextEdit,
    TextStyle, TextureOptions, TopBottomPanel, Ui,
};
use eframe::{CreationContext, Frame};

const MY_LOCAL_PATH: &str = "C:\\Users\\loren\\Desktop\\OSTs";

pub struct RustifyApp {
    path: String,
    sink: MusicPlayer,
    volume_input: f32,
    duration_slider: f32,
}

impl RustifyApp {
    pub fn new(cc: &CreationContext) -> Self {
        // repaint thread
        let ctx_clone = cc.egui_ctx.clone();
        std::thread::spawn(move || loop {
            sleep(Duration::from_millis(500));
            ctx_clone.request_repaint();
        });

        Self {
            path: MY_LOCAL_PATH.to_string(),
            sink: MusicPlayer::new(),
            volume_input: 1.0,
            //paused_input: false,
            duration_slider: 0.0,
        }
    }

    fn spawn_duration_slider(&mut self, ui: &mut Ui) {
        ui.spacing_mut().slider_width = ui.available_width();
        let response = ui.add(
            Slider::new(&mut self.duration_slider, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );
        if response.drag_stopped() {
            self.sink.jump(self.duration_slider);
        }
    }

    pub fn spawn_pause_button(&mut self, ui: &mut Ui) {
        let text = if self.sink.get_paused() { "▶" } else { "⏸" };
        let response = ui.add_sized(
            [40.0, 40.0],
            Button::new(RichText::new(text).size(20.0)).rounding(7.0),
        );
        if response.clicked() {
            self.sink.set_paused(!self.sink.get_paused())
        }
    }

    pub fn spawn_skip_button(&mut self, ui: &mut Ui) {
        let text = "⏭";
        let response = ui.add_sized(
            [40.0, 40.0],
            Button::new(RichText::new(text).size(20.0)).rounding(7.0),
        );
        if response.clicked() {
            self.sink.skip();
        }
    }
}

impl eframe::App for RustifyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let mut enable_duration_bar = true;

                if let Some(track) = self.sink.get_current_track() {
                    // get pos
                    let pos = self.sink.get_current_track_pos();
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
                    ui.label(formatted_duration(&self.sink.get_current_track_pos()));
                    // layout needed for correct expansion of the slider
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(track) = self.sink.get_current_track() {
                            ui.label(formatted_duration(&track.duration));
                        } else {
                            ui.label("00:00");
                        }
                        ui.add_enabled_ui(enable_duration_bar, |ui| {
                            self.spawn_duration_slider(ui);
                        });
                    });
                });
                // TODO: fix this alignment
                // ui.horizontal(|ui| {
                //     self.spawn_pause_button(ui);
                //     self.spawn_skip_button(ui);
                // });
                ui.columns(3, |cols| {
                    cols[0].vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label("Volume");
                            let response = ui.add(
                                Slider::new(&mut self.volume_input, 0.0..=1.0).show_value(false),
                            );
                            if response.changed() {
                                self.sink.set_volume(self.volume_input);
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
            // if ui.button("Add random track to queue").clicked() {
            //     self.append_random_from_path();
            // }
            // if ui.button("Clear queue").clicked() {
            //     self.sink.clear();
            // }
            // ui.add_space(15.0);

            ui.label("Root path");
            ui.add(
                TextEdit::singleline(&mut self.path)
                    .desired_width(f32::INFINITY)
                    .font(TextStyle::Monospace),
            );
            if ui.button("Play root").clicked() {
                let r = RootDir::new(PathBuf::from(self.path.clone()));
                self.sink.set_playlist(Rc::clone(&r.root));
            }
            ui.add_space(15.0);

            // let response = ui.add(Checkbox::new(&mut self.paused_input, "Paused"));
            // if response.changed() {
            //     self.sink.set_paused(self.paused_input);
            // }

            if let Some(track) = self.sink.get_current_track() {
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
