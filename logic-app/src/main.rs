mod app;
mod canvas;
mod glyphs;
mod hdl_ui;
mod palette;
mod theme;
mod tools;
mod verification_ui;
mod waveform;

use app::LogicLabApp;
use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("Logic Lab"),
        ..Default::default()
    };

    eframe::run_native(
        "Logic Lab",
        native_options,
        Box::new(|cc| Ok(Box::new(LogicLabApp::new(cc)))),
    )
}
