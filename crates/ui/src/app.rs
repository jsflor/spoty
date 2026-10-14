use eframe::egui;
use spoty_audio::{PlayerController, PlayerEvent};
use spoty_core::{PlaybackStatus, Queue, Track};
use spoty_storage::{DemoProvider, Library, MusicProvider, OpenDalProvider};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;

use crate::navigation::{NavigationHistory, Route};
use crate::views::{HomeView, PlaylistView, SearchView, SettingsView};

pub struct SpotyApp {
    library: Library,
    provider: Arc<dyn MusicProvider>,
    controller: PlayerController,
    event_rx: Receiver<PlayerEvent>,
    tokio_rt: Arc<tokio::runtime::Runtime>,
    nav: NavigationHistory,
    queue: Queue,
    search_query: String,
    local_dir_input: String,
}

impl SpotyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tokio_rt = Arc::new(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));
        let provider: Arc<dyn MusicProvider> = Arc::new(DemoProvider::new());

        let library = tokio_rt
            .block_on(provider.fetch_library())
            .unwrap_or_default();

        let (controller, event_rx) = PlayerController::new();

        let queue = Queue::with_tracks(library.tracks.clone(), None);

        Self {
            library,
            provider,
            controller,
            event_rx,
            tokio_rt,
            nav: NavigationHistory::new(),
            queue,
            search_query: String::new(),
            local_dir_input: String::new(),
        }
    }

    fn play_track(&mut self, track: Track) {
        if let Some(pos) = self.queue.tracks().iter().position(|t| t.id == track.id) {
            self.queue.jump_to(pos);
        }

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

    fn play_next(&mut self) {
        let repeat = self.controller.state().repeat_mode;
        if let Some(next_track) = self.queue.next(repeat).cloned() {
            self.play_track(next_track);
        }
    }

    fn play_prev(&mut self) {
        let state = self.controller.state();
        // If track played for more than 3 seconds, replay from beginning
        if state.position_secs > 3.0 {
            self.controller.seek(Duration::ZERO);
        } else if let Some(prev_track) = self.queue.prev().cloned() {
            self.play_track(prev_track);
        }
    }

    fn play_all_tracks(&mut self, tracks: Vec<Track>) {
        if let Some(first) = tracks.first().cloned() {
            self.queue.set_tracks(tracks, Some(0));
            self.play_track(first);
        }
    }

    fn switch_to_local_fs(&mut self, path: String) {
        match OpenDalProvider::new_fs(&path) {
            Ok(new_provider) => {
                let provider_arc: Arc<dyn MusicProvider> = Arc::new(new_provider);
                if let Ok(new_lib) = self.tokio_rt.block_on(provider_arc.fetch_library()) {
                    self.provider = provider_arc;
                    self.library = new_lib.clone();
                    self.queue.set_tracks(new_lib.tracks, None);
                    self.nav.push(Route::Home);
                }
            }
            Err(e) => {
                tracing::error!("Failed to mount local directory: {}", e);
            }
        }
    }

    fn switch_to_demo(&mut self) {
        let provider_arc: Arc<dyn MusicProvider> = Arc::new(DemoProvider::new());
        if let Ok(new_lib) = self.tokio_rt.block_on(provider_arc.fetch_library()) {
            self.provider = provider_arc;
            self.library = new_lib.clone();
            self.queue.set_tracks(new_lib.tracks, None);
            self.nav.push(Route::Home);
        }
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
                PlayerEvent::TrackEnded(_) => {
                    self.play_next();
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
                // Back button
                let back_enabled = self.nav.can_go_back();
                if ui
                    .add_enabled(back_enabled, egui::Button::new("◀"))
                    .on_hover_text("Go back")
                    .clicked()
                {
                    self.nav.back();
                }

                // Forward button
                let fwd_enabled = self.nav.can_go_forward();
                if ui
                    .add_enabled(fwd_enabled, egui::Button::new("▶"))
                    .on_hover_text("Go forward")
                    .clicked()
                {
                    self.nav.forward();
                }

                ui.add_space(16.0);

                let search_box = egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("What do you want to play?")
                    .desired_width(320.0);

                if ui.add(search_box).changed() {
                    if *self.nav.current() != Route::Search {
                        self.nav.push(Route::Search);
                    }
                }
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

                        let shuffle_btn_color = if self.queue.is_shuffle() {
                            egui::Color32::from_rgb(0x1e, 0xd7, 0x60)
                        } else {
                            egui::Color32::GRAY
                        };

                        if ui
                            .button(egui::RichText::new("🔀").color(shuffle_btn_color))
                            .on_hover_text("Shuffle")
                            .clicked()
                        {
                            let is_shuf = self.queue.toggle_shuffle();
                            if is_shuf != state.is_shuffle {
                                self.controller.toggle_shuffle();
                            }
                        }

                        if ui.button("⏮").on_hover_text("Previous").clicked() {
                            self.play_prev();
                        }

                        let play_btn_text = match state.status {
                            PlaybackStatus::Playing => "⏸",
                            _ => "▶",
                        };

                        if ui.button(egui::RichText::new(play_btn_text).size(18.0)).clicked() {
                            self.controller.toggle_play_pause();
                        }

                        if ui.button("⏭").on_hover_text("Next").clicked() {
                            self.play_next();
                        }

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
                ui.heading("Spoty");
                ui.add_space(8.0);

                if ui.button("🏠 Home").clicked() {
                    self.nav.push(Route::Home);
                }
                if ui.button("🔍 Search").clicked() {
                    self.nav.push(Route::Search);
                }
                if ui.button("⚙ Settings").clicked() {
                    self.nav.push(Route::Settings);
                }

                ui.add_space(16.0);
                ui.heading("Your Library");
                ui.add_space(8.0);

                for playlist in &self.library.playlists {
                    if ui.button(format!("📁 {}", playlist.title)).clicked() {
                        self.nav.push(Route::Playlist(playlist.id.clone()));
                    }
                }

                ui.add_space(12.0);
                ui.label(egui::RichText::new("Artists").strong());
                for artist in &self.library.artists {
                    ui.label(format!("👤 {}", artist.name));
                }
            });

        // Central Panel: Render Active View
        egui::CentralPanel::default().show(ctx, |ui| {
            let active_track_id = state.current_track.as_ref().map(|t| &t.id);
            let current_route = self.nav.current().clone();

            match current_route {
                Route::Home => {
                    let mut nav_action = None;
                    let mut play_action = None;
                    let mut playlist_action = None;

                    HomeView::show(
                        ui,
                        &self.library,
                        active_track_id,
                        &mut |route| nav_action = Some(route),
                        &mut |track| play_action = Some(track),
                        &mut |playlist| playlist_action = Some(playlist),
                    );

                    if let Some(route) = nav_action {
                        self.nav.push(route);
                    }
                    if let Some(track) = play_action {
                        self.play_track(track);
                    }
                    if let Some(playlist) = playlist_action {
                        let tracks: Vec<_> = playlist
                            .track_ids
                            .iter()
                            .filter_map(|id| self.library.tracks.iter().find(|t| &t.id == id).cloned())
                            .collect();
                        self.play_all_tracks(tracks);
                    }
                }
                Route::Search => {
                    let mut play_action = None;

                    SearchView::show(
                        ui,
                        &self.library,
                        &mut self.search_query,
                        active_track_id,
                        &mut |track| play_action = Some(track),
                    );

                    if let Some(track) = play_action {
                        self.play_track(track);
                    }
                }
                Route::Playlist(playlist_id) => {
                    let mut play_action = None;
                    let mut play_all_action = None;

                    PlaylistView::show(
                        ui,
                        &self.library,
                        &playlist_id,
                        active_track_id,
                        &mut |track| play_action = Some(track),
                        &mut |tracks| play_all_action = Some(tracks),
                    );

                    if let Some(track) = play_action {
                        self.play_track(track);
                    }
                    if let Some(tracks) = play_all_action {
                        self.play_all_tracks(tracks);
                    }
                }
                Route::Album(_) => {
                    ui.label("Album view");
                }
                Route::Settings => {
                    let provider_name = self.provider.display_name().to_string();
                    let mut switch_fs_action = None;
                    let mut switch_demo_action = false;

                    SettingsView::show(
                        ui,
                        &provider_name,
                        &mut self.local_dir_input,
                        &mut |path| switch_fs_action = Some(path),
                        &mut || switch_demo_action = true,
                    );

                    if let Some(path) = switch_fs_action {
                        self.switch_to_local_fs(path);
                    }
                    if switch_demo_action {
                        self.switch_to_demo();
                    }
                }
            }
        });
    }
}
