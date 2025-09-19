use crate::frontend::eframe_app::EmptyDisplayMessage;
use crate::frontend::App;
use crate::music_dir_creation_error::MusicDirCreationError;
use eframe::egui::{CentralPanel, Context, Image};

impl App {
    pub(crate) fn spawn_image_central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            let texture_handle = self
                .current_texture
                .as_ref()
                .unwrap_or(&self.default_texture);
            ui.centered_and_justified(|ui| {
                let max_size = ui.available_size();
                ui.add(Image::new(texture_handle).max_size(max_size).rounding(10.0));
            });
        });
    }

    pub(crate) fn spawn_empty_central_panel(
        &mut self,
        ctx: &Context,
        message: EmptyDisplayMessage,
    ) {
        let text: &str = match message {
            EmptyDisplayMessage::SelectFolder => "Select a folder",
            EmptyDisplayMessage::Error(e) => match e {
                MusicDirCreationError::NotFound => "Error: path not found",
                MusicDirCreationError::NotDir => "Error: selected path is not a folder",
                MusicDirCreationError::Empty => "Error: no .mp3 files found inside the selected folder and its relative sub-folders",
                MusicDirCreationError::Unknown => "An unknown error occurred",
            }
        };
        CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(text);
            });
        });
    }

    pub(crate) fn spawn_loading_central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label("Loading tracks...");
            });
        });
    }
}
