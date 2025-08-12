use crate::frontend::eframe_app::{PauseButtonAction, PauseButtonState, ProgressBarState};
use crate::frontend::App;
use crate::messages::Request;
use eframe::egui::{Button, RichText, Slider, Ui};

impl App {
    pub(crate) fn spawn_duration_slider(&mut self, ui: &mut Ui) {
        ui.spacing_mut().slider_width = ui.available_width();

        let mut progress_fraction = match self.get_current_track_duration() {
            None => 0.0,
            Some(dur) => self.progress.as_secs_f32() / dur.as_secs_f32(),
        };

        let enabled = self.progress_bar_state == ProgressBarState::Active;

        let response = ui.add_enabled(
            enabled,
            Slider::new(&mut progress_fraction, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );
        if response.drag_stopped() {
            self.progress_bar_state = ProgressBarState::WaitingForJump;
            self.req_sender
                .send(Request::JumpToFraction(progress_fraction))
                .unwrap()
        }
    }

    pub fn spawn_pause_button(&mut self, ui: &mut Ui) {
        let text = match self.pause_button_action {
            PauseButtonAction::Pause => "⏸",
            PauseButtonAction::Play => "▶",
        };
        let response = ui.add_sized(
            [40.0, 40.0],
            Button::new(RichText::new(text).size(20.0)).rounding(7.0),
        );
        if response.clicked() {
            match self.pause_button_action {
                PauseButtonAction::Pause => {
                    println!("UI: Pause sent");
                    self.req_sender.send(Request::Pause).unwrap()
                }
                PauseButtonAction::Play => {
                    println!("UI: Play sent");
                    self.req_sender.send(Request::Play).unwrap()
                }
            }
            self.pause_button_state = PauseButtonState::WaitingForEvent;
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
