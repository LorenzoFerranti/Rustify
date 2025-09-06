use crate::frontend::App;
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
}
