use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{
    Align, CentralPanel, Color32, ColorImage, Context, Image, Layout, RichText, Slider, TextEdit,
    TextStyle, TextureHandle, TextureOptions, TopBottomPanel,
};
use eframe::{CreationContext, Frame};

use crate::messages::{Event, Request};
use crate::settings::Settings;
use crate::track_metadata::TrackMetaData;

const MY_LOCAL_PATH: &str = "C:\\Users\\loren\\Desktop\\OSTs";

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ProgressBarState {
    Active,
    Disabled,
    WaitingForJump,
}

pub(crate) enum PauseButtonState {
    Active,
    Disabled,
    WaitingForEvent,
}

pub(crate) enum PauseButtonAction {
    Pause,
    Play,
}

pub struct App {
    root_music_path_input: String,
    volume_input: f32,
    pub(crate) progress: Duration,
    pub(crate) progress_bar_state: ProgressBarState,
    pub(crate) pause_button_action: PauseButtonAction,
    pub(crate) pause_button_state: PauseButtonState,
    current_track_metadata: Option<Arc<TrackMetaData>>,
    current_texture: Option<TextureHandle>,
    pub(crate) req_sender: Sender<Request>,
    event_receiver: Receiver<Event>,
}

impl App {
    pub fn new(
        cc: &CreationContext,
        initial_settings: Settings,
        req_sender: Sender<Request>,
        event_receiver: Receiver<Event>,
    ) -> Self {
        let ctx_clone = cc.egui_ctx.clone();

        req_sender.send(Request::ProvideContext(ctx_clone)).unwrap();

        Self {
            root_music_path_input: initial_settings.root_music_path,
            volume_input: initial_settings.volume,
            progress: Duration::from_secs(0),
            progress_bar_state: ProgressBarState::Disabled,
            pause_button_action: PauseButtonAction::Play,
            pause_button_state: PauseButtonState::Disabled,
            current_track_metadata: None,
            current_texture: None,
            req_sender,
            event_receiver,
        }
    }

    fn read_events(&mut self, ctx: &Context) {
        let events: Vec<_> = self.event_receiver.try_iter().collect();
        for e in events {
            match e {
                Event::NewTrackPlaying(metadata) => {
                    println!("Frontend: New metadata! {metadata:?}");
                    self.update_metadata(ctx, metadata);
                    self.progress_bar_state = ProgressBarState::Active;
                    self.pause_button_action = PauseButtonAction::Pause;
                    self.pause_button_state = PauseButtonState::Active;
                }
                Event::ProgressUpdate(d) => match self.progress_bar_state {
                    ProgressBarState::Active => {
                        self.set_progress_rounded(d);
                    }
                    ProgressBarState::Disabled => {}
                    ProgressBarState::WaitingForJump => {}
                },
                Event::JumpedTo(d) => match self.progress_bar_state {
                    ProgressBarState::Active => {
                        self.set_progress_rounded(d);
                    }
                    ProgressBarState::Disabled => {}
                    ProgressBarState::WaitingForJump => {
                        self.set_progress_rounded(d);
                        self.progress_bar_state = ProgressBarState::Active;
                    }
                },
                Event::NowPlaying => {
                    self.pause_button_action = PauseButtonAction::Pause;
                    match self.pause_button_state {
                        PauseButtonState::Active => {}
                        PauseButtonState::Disabled | PauseButtonState::WaitingForEvent => {
                            self.pause_button_state = PauseButtonState::Active;
                        }
                    }
                }
                Event::NowPaused => {
                    self.pause_button_action = PauseButtonAction::Play;
                    match self.pause_button_state {
                        PauseButtonState::Active => {}
                        PauseButtonState::Disabled | PauseButtonState::WaitingForEvent => {
                            self.pause_button_state = PauseButtonState::Active;
                        }
                    }
                }
                Event::NewSettings(s) => {
                    self.volume_input = s.volume;
                    self.root_music_path_input = s.root_music_path;
                }
            }
        }
    }

    fn set_progress_rounded(&mut self, d: Duration) {
        let millis = d.as_millis();
        let rounded_millis = if millis > 60_000 {
            // long track -> round to seconds
            (((millis + 500) / 1000) * 1000) as u64
        } else {
            // short track -> round to half second
            (((millis + 250) / 500) * 500) as u64
        };
        self.progress = Duration::from_millis(rounded_millis);
    }

    fn update_metadata(&mut self, ctx: &Context, metadata: Option<Arc<TrackMetaData>>) {
        match metadata {
            None => {
                self.current_track_metadata = None;
                self.current_texture = None;
            }
            Some(metadata) => {
                // TODO: find a way to avoid cloning
                match metadata.image.clone() {
                    None => {
                        self.current_texture = None;
                    }
                    Some(image) => {
                        self.current_texture =
                            Some(ctx.load_texture("my_texture", image, TextureOptions::default()));
                    }
                }
                self.current_track_metadata = Some(metadata);
            }
        }
    }

    pub(crate) fn get_current_track_duration(&self) -> Option<Duration> {
        let metadata = self.current_track_metadata.as_ref()?;
        metadata.duration.clone()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.read_events(ctx);

        TopBottomPanel::bottom("bottom").show(ctx, |ui| {
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
                    // cols[2].vertical_centered(|ui| self.spawn_skip_button(ui));
                    cols[2].with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        self.spawn_skip_button(ui);
                    });
                });

                ui.add_space(5.0);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.label("Root path");
            ui.add(
                TextEdit::singleline(&mut self.root_music_path_input)
                    .desired_width(f32::INFINITY)
                    .font(TextStyle::Monospace),
            );
            if ui.button("Play root").clicked() {
                self.req_sender
                    .send(Request::ChangeRoot(PathBuf::from(
                        self.root_music_path_input.clone(),
                    )))
                    .unwrap();
            }
            ui.add_space(15.0);

            if let Some(texture) = &self.current_texture {
                ui.centered_and_justified(|ui| {
                    let max_size = ui.available_size();
                    ui.add(Image::new(texture).max_size(max_size).rounding(10.0));
                });
            } else {
                ui.label("NO image");
            }
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
