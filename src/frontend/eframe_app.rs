use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::frontend::eframe_app::EmptyDisplayMessage::Error;
use crate::image_utils;
use crate::messages::{Event, Request};
use crate::music_dir_creation_error::MusicDirCreationError;
use crate::settings::Settings;
use crate::track_metadata::TrackMetaData;
use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{CentralPanel, Context, TextureHandle, TextureOptions};
use eframe::{CreationContext, Frame};

const DEFAULT_TEXTURE_PATH: &str = "assets/cover.png";

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum AppState {
    Empty(EmptyDisplayMessage),
    LoadingNewMusicDir,
    Playing(ProgressBarState, PauseButtonState, PauseButtonAction),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum EmptyDisplayMessage {
    SelectFolder,
    Error(MusicDirCreationError),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ProgressBarState {
    Active,
    WaitingForJump,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum PauseButtonState {
    Active,
    WaitingForEvent,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum PauseButtonAction {
    Pause,
    Play,
}

pub struct App {
    pub(crate) root_music_path_input: String,
    pub(crate) volume_input: f32,
    pub(crate) progress: Duration,
    pub(crate) state: AppState,
    pub(crate) current_track_metadata: Option<Arc<TrackMetaData>>,
    pub(crate) current_texture: Option<TextureHandle>,
    pub(crate) default_texture: TextureHandle,
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

        let default_texture = load_default_texture(&cc.egui_ctx);

        Self {
            root_music_path_input: initial_settings.root_music_path,
            volume_input: initial_settings.volume,
            progress: Duration::from_secs(0),
            state: AppState::Empty(EmptyDisplayMessage::SelectFolder),
            current_track_metadata: None,
            current_texture: None,
            default_texture,
            req_sender,
            event_receiver,
        }
    }

    fn read_events(&mut self, ctx: &Context) {
        let events: Vec<_> = self.event_receiver.try_iter().collect();
        for e in events {
            match e {
                Event::NewTrackPlaying(metadata) => {
                    let was_some: bool = metadata.is_some();
                    self.update_metadata(ctx, metadata);
                    match self.state {
                        AppState::Empty(_) => {} // happens during music dir loading error
                        AppState::LoadingNewMusicDir => {
                            if was_some {
                                self.state = AppState::Playing(
                                    ProgressBarState::Active,
                                    PauseButtonState::Active,
                                    PauseButtonAction::Pause,
                                );
                            }
                        }
                        AppState::Playing(_, _, _) => {
                            self.state = AppState::Playing(
                                ProgressBarState::Active,
                                PauseButtonState::Active,
                                PauseButtonAction::Pause,
                            );
                        }
                    }
                }
                Event::ProgressUpdate(d) => match self.state {
                    AppState::Empty(_) => unreachable!(),
                    AppState::LoadingNewMusicDir => unreachable!(),
                    AppState::Playing(progress_bar_state, _, _) => match progress_bar_state {
                        ProgressBarState::Active => {
                            self.set_progress_rounded(d);
                        }
                        ProgressBarState::WaitingForJump => {}
                    },
                },
                Event::JumpedTo(d) => match self.state {
                    AppState::Empty(_) => unreachable!(),
                    AppState::LoadingNewMusicDir => unreachable!(),
                    AppState::Playing(progress_bar_state, x, y) => match progress_bar_state {
                        ProgressBarState::Active => {
                            self.set_progress_rounded(d);
                        }
                        ProgressBarState::WaitingForJump => {
                            self.set_progress_rounded(d);
                            self.state = AppState::Playing(ProgressBarState::Active, x, y);
                        }
                    },
                },
                Event::NowPlaying => match self.state {
                    AppState::Empty(_) => unreachable!(),
                    AppState::LoadingNewMusicDir => {}
                    AppState::Playing(x, _, _) => {
                        self.state =
                            AppState::Playing(x, PauseButtonState::Active, PauseButtonAction::Pause)
                    }
                },
                Event::NowPaused => match self.state {
                    AppState::Empty(_) => unreachable!(),
                    AppState::LoadingNewMusicDir => unreachable!(),
                    AppState::Playing(x, _, _) => {
                        self.state =
                            AppState::Playing(x, PauseButtonState::Active, PauseButtonAction::Play)
                    }
                },
                Event::NewSettings(s) => {
                    self.volume_input = s.volume;
                    self.root_music_path_input = s.root_music_path;
                }
                Event::DirError(e) => {
                    self.state = AppState::Empty(Error(e));
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
                        self.current_texture = Some(ctx.load_texture(
                            "current_texture",
                            image,
                            TextureOptions::default(),
                        ));
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
        self.spawn_path_top_panel(ctx);
        match self.state {
            AppState::Empty(message) => {
                self.spawn_empty_central_panel(ctx, message);
            }
            AppState::LoadingNewMusicDir => {
                self.spawn_loading_central_panel(ctx);
            }
            AppState::Playing(_, _, _) => {
                self.spawn_track_bottom_panel(ctx);
                if self.current_track_metadata.is_some() {
                    self.spawn_image_central_panel(ctx);
                } else {
                    CentralPanel::default().show(ctx, |_| {});
                }
            }
        }
    }
}

fn load_default_texture(ctx: &Context) -> TextureHandle {
    let default_texture = image_utils::load_color_image(Path::new(DEFAULT_TEXTURE_PATH)).unwrap();
    ctx.load_texture(
        "default_texture",
        default_texture,
        TextureOptions::default(),
    )
}
