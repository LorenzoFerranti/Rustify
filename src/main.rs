mod app;
mod helper;
mod sink_wrapper;
mod root_dir;

use eframe::egui::ViewportBuilder;

use crate::app::RustifyApp;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size((600.0, 600.0)).with_min_inner_size((400.0, 300.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Rustify",
        native_options,
        Box::new(|cc| Ok(Box::new(RustifyApp::new(cc)))),
    )
}
