use crate::frontend::eframe_app::AppState;
use crate::frontend::App;
use crate::messages::Request;
use eframe::egui::{Align, Button, Context, Layout, TextEdit, TextStyle, TopBottomPanel, Vec2};
use std::path::PathBuf;

impl App {
    pub(crate) fn spawn_path_top_panel(&mut self, ctx: &Context) {
        TopBottomPanel::top("path").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.label("Music folder");
            ui.add_space(5.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), 50.0),
                Layout::right_to_left(Align::TOP),
                |ui| {
                    let response = ui.add_enabled(
                        self.state != AppState::LoadingNewMusicDir,
                        Button::new("🔀"),
                    );

                    if response.clicked() {
                        self.req_sender
                            .send(Request::ChangeRoot(PathBuf::from(
                                self.root_music_path_input.clone(),
                            )))
                            .unwrap();
                        self.state = AppState::LoadingNewMusicDir;
                    }
                    ui.add(
                        TextEdit::singleline(&mut self.root_music_path_input)
                            .desired_width(f32::INFINITY)
                            .font(TextStyle::Monospace),
                    );
                },
            );
            ui.add_space(5.0);
        });
    }
}
