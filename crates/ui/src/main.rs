mod app;
mod theme;

use app::SpotyApp;
use eframe::egui;

fn main() -> eframe::Result {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 720.0])
            .with_min_inner_size([700.0, 500.0])
            .with_title("Spoty — Open Self-Hosted Music Player"),
        ..Default::default()
    };

    eframe::run_native(
        "Spoty",
        native_options,
        Box::new(|cc| {
            theme::setup_spotify_theme(&cc.egui_ctx);
            Ok(Box::new(SpotyApp::new(cc)))
        }),
    )
}
