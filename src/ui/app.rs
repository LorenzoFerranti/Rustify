
use std::{cmp, thread};
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

pub(crate) const DATA_RELATIVE_PATH: &str = "data.json";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RustifyOptions {
    root_path: String,
    pub(crate) volume: f32,
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
    pub(crate) options: RustifyOptions,
    pub(crate) music_player: MusicPlayer,
    pub(crate) duration_slider: f32,
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
        let options = Self::get_options();

        Self {
            options,
            music_player: MusicPlayer::new(),
            duration_slider: 0.0,
        }
    }
}

impl eframe::App for RustifyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {

        self.spawn_track_bottom_panel(ctx);

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
                    // TODO: do not call this every frame!
                    let texture =
                        ctx.load_texture("my_texture", color_image, TextureOptions::default());
                    ui.centered_and_justified(|ui| {
                        let max_size = ui.available_size();
                        ui.add(Image::new(&texture)
                            .max_size(max_size)
                            .rounding(10.0)
                        );
                    });
                } else {
                    ui.label("NO image");
                }
            }
        });
    }
}
