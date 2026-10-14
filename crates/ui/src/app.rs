use eframe::egui;
use spoty_audio::{PlayerController, PlayerEvent};
use spoty_core::{PlaybackStatus, Track};
use spoty_storage::{DemoProvider, Library, MusicProvider};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;

pub struct SpotyApp {
    library: Library,
    provider: Arc<dyn MusicProvider>,
    controller: PlayerController,
    event_rx: Receiver<PlayerEvent>,
    tokio_rt: Arc<tokio::runtime::Runtime>,
    search_query: String,
}

impl SpotyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tokio_rt = Arc::new(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));
        let provider: Arc<dyn MusicProvider> = Arc::new(DemoProvider::new());

        let library = tokio_rt
            .block_on(provider.fetch_library())
            .unwrap_or_default();

        let (controller, event_rx) = PlayerController::new();

        Self {
            library,
            provider,
            controller,
            event_rx,
            tokio_rt,
            search_query: String::new(),
        }
    }

    fn play_track(&self, track: Track) {
        let provider = self.provider.clone();
        let controller = self.controller.clone();
        let track_clone = track.clone();

        self.tokio_rt.spawn(async move {
            match provider.stream_track(&track_clone.id).await {
                Ok(mut stream) => {
                    let mut bytes = Vec::new();
                    if let Ok(_) = stream.read_to_end(&mut bytes).await {
                        controller.load_track(track_clone, bytes);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to stream track: {}", e);
                }
            }
        });
    }
}

impl eframe::App for SpotyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain incoming background audio worker events
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                PlayerEvent::StateChanged(state) => {
                    self.controller.update_cached_state(state);
                    ctx.request_repaint();
                }
                PlayerEvent::TrackEnded(track) => {
                    tracing::info!("Track finished: {}", track.title);
                }
                PlayerEvent::Error(err) => {
                    tracing::error!("Audio error: {}", err);
                }
            }
        }

        let state = self.controller.state();

        // Top Panel: Navigation & Search
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("◀").size(16.0));
                ui.label(egui::RichText::new("▶").size(16.0));
                ui.add_space(16.0);

                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search by title, artist, or album...")
                        .desired_width(320.0),
                );
            });
        });

        // Bottom Panel: Persistent Player
        egui::TopBottomPanel::bottom("player_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left: Current track info
                ui.allocate_ui(egui::vec2(220.0, 40.0), |ui| {
                    if let Some(track) = &state.current_track {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&track.title).strong().color(egui::Color32::WHITE));
                            ui.label(egui::RichText::new(&track.artist).small().color(egui::Color32::GRAY));
                        });
                    } else {
                        ui.label(egui::RichText::new("No track playing").weak());
                    }
                });

                // Center: Playback controls and timeline
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);

                        if ui.button("🔀").on_hover_text("Shuffle").clicked() {
                            self.controller.toggle_shuffle();
                        }
                        if ui.button("⏮").on_hover_text("Previous").clicked() {
                            // Replay from beginning
                            self.controller.seek(Duration::ZERO);
                        }

                        let play_btn_text = match state.status {
                            PlaybackStatus::Playing => "⏸",
                            _ => "▶",
                        };

                        if ui.button(egui::RichText::new(play_btn_text).size(18.0)).clicked() {
                            self.controller.toggle_play_pause();
                        }

                        if ui.button("⏭").on_hover_text("Next").clicked() {}

                        let repeat_icon = match state.repeat_mode {
                            spoty_core::RepeatMode::Off => "🔁",
                            spoty_core::RepeatMode::All => "🔁 (All)",
                            spoty_core::RepeatMode::One => "🔂 (1)",
                        };
                        if ui.button(repeat_icon).on_hover_text("Repeat").clicked() {
                            self.controller.cycle_repeat_mode();
                        }
                    });

                    // Timeline progress bar
                    ui.horizontal(|ui| {
                        let current_mins = (state.position_secs / 60.0).floor() as u32;
                        let current_secs = (state.position_secs % 60.0).floor() as u32;
                        ui.label(format!("{}:{:02}", current_mins, current_secs));

                        let mut pos = state.position_secs;
                        let max_duration = if state.duration_secs > 0.0 { state.duration_secs } else { 1.0 };

                        let slider = egui::Slider::new(&mut pos, 0.0..=max_duration)
                            .show_value(false)
                            .trailing_fill(true);

                        if ui.add(slider).changed() {
                            self.controller.seek(Duration::from_secs_f64(pos));
                        }

                        let total_mins = (state.duration_secs / 60.0).floor() as u32;
                        let total_secs = (state.duration_secs % 60.0).floor() as u32;
                        ui.label(format!("{}:{:02}", total_mins, total_secs));
                    });
                });

                // Right: Volume control
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut volume = state.volume;
                    if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0).show_value(false)).changed() {
                        self.controller.set_volume(volume);
                    }

                    let mute_icon = if state.is_muted { "🔇" } else { "🔊" };
                    if ui.button(mute_icon).clicked() {
                        self.controller.toggle_mute();
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

                ui.add_space(12.0);
                ui.label(egui::RichText::new("Albums").strong());
                for album in &self.library.albums {
                    ui.label(format!("💿 {}", album.title));
                }
            });

        // Central Panel: Catalog & Track List
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Discover Music");
            ui.add_space(12.0);

            let filter = self.search_query.to_lowercase();
            let tracks: Vec<_> = self
                .library
                .tracks
                .iter()
                .filter(|t| {
                    filter.is_empty()
                        || t.title.to_lowercase().contains(&filter)
                        || t.artist.to_lowercase().contains(&filter)
                })
                .collect();

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

                    for (index, track) in tracks.iter().enumerate() {
                        let is_current = state
                            .current_track
                            .as_ref()
                            .map(|t| t.id == track.id)
                            .unwrap_or(false);

                        let title_color = if is_current {
                            egui::Color32::from_rgb(0x1e, 0xd7, 0x60) // Spotify green for playing
                        } else {
                            egui::Color32::WHITE
                        };

                        ui.label(format!("{}", index + 1));
                        ui.label(egui::RichText::new(&track.title).color(title_color));
                        ui.label(&track.artist);
                        ui.label(track.album.as_deref().unwrap_or("-"));
                        ui.label(track.formatted_duration());

                        let btn_text = if is_current && state.status == PlaybackStatus::Playing {
                            "⏸ Pause"
                        } else {
                            "▶ Play"
                        };

                        if ui.button(btn_text).clicked() {
                            if is_current {
                                self.controller.toggle_play_pause();
                            } else {
                                self.play_track((*track).clone());
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    }
}
