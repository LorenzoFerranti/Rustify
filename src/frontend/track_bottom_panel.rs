use crate::frontend::App;
use crate::messages::Request;
use eframe::egui::{Align, Color32, Context, Layout, RichText, Slider, TopBottomPanel};
use std::time::Duration;

impl App {
    pub(crate) fn spawn_track_bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("track").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let mut enable_duration_bar = true;

                if let Some(metadata) = &self.current_track_metadata {
                    // name, artist and album
                    ui.add_space(5.0);
                    ui.horizontal_wrapped(|ui| {
                        let text = RichText::new(&metadata.name).color(Color32::WHITE);
                        ui.heading(text);
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{} - {}", &metadata.artist, &metadata.album));
                    });
                } else {
                    enable_duration_bar = false;
                }

                ui.add_space(5.0);

                // slider
                ui.horizontal(|ui| {
                    // TODO: change
                    ui.label(formatted_duration(&self.progress));
                    // layout needed for correct expansion of the slider
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        match self.get_current_track_duration() {
                            None => {
                                ui.label("--:--");
                            }
                            Some(d) => {
                                ui.label(formatted_duration(&d));
                            }
                        }

                        ui.add_enabled_ui(enable_duration_bar, |ui| {
                            self.spawn_duration_slider(ui);
                        });
                    });
                });
                ui.add_space(10.0);
                ui.columns(3, |cols| {
                    cols[0].vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔊").size(30.0));
                            let response = ui.add(
                                Slider::new(&mut self.volume_input, 0.0..=1.0).show_value(false),
                            );
                            if response.changed() {
                                self.req_sender
                                    .send(Request::SetVolume(self.volume_input))
                                    .unwrap();
                            }
                        });
                    });
                    cols[1].vertical_centered(|ui| self.spawn_pause_button(ui));
                    cols[2].with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        self.spawn_skip_button(ui);
                    });
                });

                ui.add_space(5.0);
            });
        });
    }
}

pub fn formatted_duration(d: &Duration) -> String {
    let tot = d.as_secs();
    let sec = tot % 60;
    let min = tot / 60;
    let mut sec_padding = "".to_string();
    let mut min_padding = "".to_string();
    if sec < 10 {
        sec_padding.push('0');
    }
    if min < 10 {
        min_padding.push('0');
    }
    format!("{}{}:{}{}", min_padding, min, sec_padding, sec)
}
