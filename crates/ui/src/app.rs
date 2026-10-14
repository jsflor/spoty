use eframe::egui;
use spoty_audio::{AudioEngine, PlayerCommand};
use spoty_core::Track;
use spoty_storage::{DemoProvider, Library, MusicProvider};

pub struct SpotyApp {
    library: Library,
    engine: AudioEngine,
    search_query: String,
    selected_track: Option<Track>,
}

impl SpotyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize with the built-in demo catalog
        let provider = DemoProvider::new();
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let library = rt
            .block_on(provider.fetch_library())
            .unwrap_or_default();

        Self {
            library,
            engine: AudioEngine::new(),
            search_query: String::new(),
            selected_track: None,
        }
    }
}

impl eframe::App for SpotyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top Panel: Navigation & Search
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("◀").size(16.0));
                ui.label(egui::RichText::new("▶").size(16.0));
                ui.add_space(16.0);

                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("What do you want to play?")
                        .desired_width(300.0),
                );
            });
        });

        // Bottom Panel: Persistent Player
        egui::TopBottomPanel::bottom("player_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left: Current track info
                ui.horizontal(|ui| {
                    if let Some(track) = &self.engine.state().current_track {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&track.title).strong().color(egui::Color32::WHITE));
                            ui.label(egui::RichText::new(&track.artist).small().color(egui::Color32::GRAY));
                        });
                    } else {
                        ui.label(egui::RichText::new("No track playing").weak());
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Right: Volume control
                    let mut volume = self.engine.state().volume;
                    if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0).show_value(false)).changed() {
                        self.engine.handle_command(PlayerCommand::SetVolume(volume));
                    }
                    ui.label("🔊");

                    ui.add_space(20.0);

                    // Center playback controls
                    let status_text = match self.engine.state().status {
                        spoty_core::PlaybackStatus::Playing => "⏸ Pause",
                        _ => "▶ Play",
                    };

                    if ui.button(status_text).clicked() {
                        self.engine.handle_command(PlayerCommand::TogglePlayPause);
                    }
                });
            });
        });

        // Left Panel: "Your Library"
        egui::SidePanel::left("library_sidebar")
            .resizable(true)
            .default_width(220.0)
            .width_range(160.0..=360.0)
            .show(ctx, |ui| {
                ui.heading("Your Library");
                ui.add_space(8.0);

                for playlist in &self.library.playlists {
                    ui.label(format!("📁 {}", playlist.title));
                }

                ui.add_space(12.0);
                ui.label(egui::RichText::new("Artists").strong());
                for artist in &self.library.artists {
                    ui.label(format!("👤 {}", artist.name));
                }
            });

        // Central Panel: Catalog & Track List
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Discover Music");
            ui.add_space(12.0);

            egui::Grid::new("track_grid")
                .striped(true)
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("#").strong());
                    ui.label(egui::RichText::new("Title").strong());
                    ui.label(egui::RichText::new("Artist").strong());
                    ui.label(egui::RichText::new("Album").strong());
                    ui.label(egui::RichText::new("Duration").strong());
                    ui.label(egui::RichText::new("Action").strong());
                    ui.end_row();

                    for (index, track) in self.library.tracks.iter().enumerate() {
                        ui.label(format!("{}", index + 1));
                        ui.label(&track.title);
                        ui.label(&track.artist);
                        ui.label(track.album.as_deref().unwrap_or("-"));
                        ui.label(track.formatted_duration());

                        if ui.button("▶ Play").clicked() {
                            self.selected_track = Some(track.clone());
                            self.engine.handle_command(PlayerCommand::LoadTrack(track.clone()));
                        }
                        ui.end_row();
                    }
                });
        });
    }
}
