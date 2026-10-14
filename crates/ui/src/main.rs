mod app;
mod config;
mod navigation;
mod theme;
mod views;
mod widgets;

use app::SpotyApp;
use eframe::egui;

fn main() -> eframe::Result {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 780.0])
            .with_min_inner_size([740.0, 540.0])
            .with_title("Spoty — Open Self-Hosted Music Player"),
        ..Default::default()
    };

    eframe::run_native(
        "Spoty",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            theme::setup_spotify_theme(&cc.egui_ctx);
            Ok(Box::new(SpotyApp::new(cc)))
        }),
    )
}
