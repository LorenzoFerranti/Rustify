use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{
    CentralPanel, Context, Image,
    TextureHandle, TextureOptions,
};
use eframe::{CreationContext, Frame};

use crate::messages::{Event, Request};
use crate::settings::Settings;
use crate::track_metadata::TrackMetaData;

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
    pub(crate) root_music_path_input: String,
    pub(crate) volume_input: f32,
    pub(crate) progress: Duration,
    pub(crate) progress_bar_state: ProgressBarState,
    pub(crate) pause_button_action: PauseButtonAction,
    pub(crate) pause_button_state: PauseButtonState,
    pub(crate) current_track_metadata: Option<Arc<TrackMetaData>>,
    pub(crate) current_texture: Option<TextureHandle>,
    pub(crate) req_sender: Sender<Request>,
    pub(crate) event_receiver: Receiver<Event>,
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

        self.spawn_track_bottom_panel(ctx);

        self.spawn_path_top_panel(ctx);

        CentralPanel::default().show(ctx, |ui| {
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
