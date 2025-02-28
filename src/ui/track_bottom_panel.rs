
use eframe::egui::{Align, Button, Color32, Context, Layout, RichText, Slider, TopBottomPanel, Ui};
use crate::helper;
use crate::ui::app::RustifyApp;

impl RustifyApp {
    pub(crate) fn spawn_track_bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("track_bottom_panel").show(ctx, |ui| {
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

    fn spawn_pause_button(&mut self, ui: &mut Ui) {
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

    fn spawn_skip_button(&mut self, ui: &mut Ui) {
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