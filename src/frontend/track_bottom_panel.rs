use crate::frontend::App;
use crate::messages::Request;
use eframe::egui::{Align, Button, Color32, Context, Layout, RichText, Slider, TopBottomPanel, Ui};
use std::time::Duration;
use crate::frontend::eframe_app::{AppState, PauseButtonAction, PauseButtonState, ProgressBarState};

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

    pub(crate) fn spawn_duration_slider(&mut self, ui: &mut Ui) {
        ui.spacing_mut().slider_width = ui.available_width();

        let current_duration = self.get_current_track_duration();

        let mut progress_fraction = match &current_duration {
            None => 0.0,
            Some(dur) => self.progress.as_secs_f32() / dur.as_secs_f32(),
        };

        let mut enabled = match self.state {
            AppState::Empty => unreachable!(),
            AppState::LoadingNewMusicDir => unreachable!(),
            AppState::Playing(pbs, _, _) => pbs == ProgressBarState::Active,
        };
        enabled &= self.get_current_track_duration() != None;

        let response = ui.add_enabled(
            enabled,
            Slider::new(&mut progress_fraction, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );
        if response.drag_stopped() {
            match self.state {
                AppState::Empty => unreachable!(),
                AppState::LoadingNewMusicDir => unreachable!(),
                AppState::Playing(_, x, y) => {
                    self.state = AppState::Playing(ProgressBarState::WaitingForJump, x, y)
                }
            };
            self.req_sender
                .send(Request::JumpToFraction(progress_fraction))
                .unwrap()
        }
    }

    pub fn spawn_pause_button(&mut self, ui: &mut Ui) {
        let text = match self.state {
            AppState::Empty => unreachable!(),
            AppState::LoadingNewMusicDir => unreachable!(),
            AppState::Playing(_, _, pba) => match pba {
                PauseButtonAction::Pause => "⏸",
                PauseButtonAction::Play => "▶",
            }
        };

        let response = ui.add_sized(
            [40.0, 40.0],
            Button::new(RichText::new(text).size(20.0)).rounding(7.0),
        );
        if response.clicked() {

            match self.state {
                AppState::Empty => unreachable!(),
                AppState::LoadingNewMusicDir => unreachable!(),
                AppState::Playing(x, _, pba) => {
                    match pba {
                        PauseButtonAction::Pause => {
                            println!("UI: Pause sent");
                            self.req_sender.send(Request::Pause).unwrap()
                        }
                        PauseButtonAction::Play => {
                            println!("UI: Play sent");
                            self.req_sender.send(Request::Play).unwrap()
                        }
                    }
                    self.state = AppState::Playing(x, PauseButtonState::WaitingForEvent, pba);
                }
            };
        }
    }

    pub fn spawn_skip_button(&mut self, ui: &mut Ui) {
        let text = "⏭";
        let response = ui.add_sized(
            [40.0, 40.0],
            Button::new(RichText::new(text).size(20.0)).rounding(7.0),
        );
        if response.clicked() {
            self.req_sender.send(Request::Skip).unwrap();
        }
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
