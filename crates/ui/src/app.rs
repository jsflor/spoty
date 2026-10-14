use eframe::egui;
use spoty_audio::{PlayerController, PlayerEvent};
use spoty_core::{PlaybackStatus, Queue, Track};
use spoty_storage::{DemoProvider, Library, MusicProvider, OpenDalProvider, S3Config, SpotifyConfig, SpotifyProvider, SubsonicConfig, SubsonicProvider};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;

use crate::config::{ProviderType, SpotyConfig};
use crate::navigation::{NavigationHistory, Route};
use crate::views::{HomeView, PlaylistView, SearchView, SettingsView};
use crate::widgets::VisualizerWidget;

pub struct SpotyApp {
    library: Library,
    provider: Arc<dyn MusicProvider>,
    controller: PlayerController,
    event_rx: Receiver<PlayerEvent>,
    tokio_rt: Arc<tokio::runtime::Runtime>,
    nav: NavigationHistory,
    queue: Queue,
    config: SpotyConfig,
    search_query: String,
    visualizer_levels: [f32; 16],
}

impl SpotyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tokio_rt = Arc::new(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));
        let config = SpotyConfig::load();

        let provider: Arc<dyn MusicProvider> = match config.active_provider {
            ProviderType::LocalFs if !config.local_path.is_empty() => {
                match OpenDalProvider::new_fs(&config.local_path) {
                    Ok(p) => Arc::new(p),
                    Err(_) => Arc::new(DemoProvider::new()),
                }
            }
            ProviderType::S3 if !config.s3.bucket.is_empty() => {
                match OpenDalProvider::new_s3(config.s3.clone()) {
                    Ok(p) => Arc::new(p),
                    Err(_) => Arc::new(DemoProvider::new()),
                }
            }
            ProviderType::Subsonic if !config.subsonic.server_url.is_empty() => {
                Arc::new(SubsonicProvider::new(config.subsonic.clone()))
            }
            ProviderType::Spotify if !config.spotify.client_id.is_empty() => {
                Arc::new(SpotifyProvider::new(config.spotify.clone()))
            }
            _ => Arc::new(DemoProvider::new()),
        };

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
            config,
            search_query: String::new(),
            visualizer_levels: [0.0; 16],
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

    fn switch_provider(&mut self, provider: Arc<dyn MusicProvider>, p_type: ProviderType) {
        if let Ok(new_lib) = self.tokio_rt.block_on(provider.fetch_library()) {
            self.provider = provider;
            self.library = new_lib.clone();
            self.queue.set_tracks(new_lib.tracks, None);
            self.config.active_provider = p_type;
            let _ = self.config.save();
            self.nav.push(Route::Home);
        }
    }

    fn switch_to_local_fs(&mut self, path: String) {
        match OpenDalProvider::new_fs(&path) {
            Ok(p) => {
                self.config.local_path = path;
                self.switch_provider(Arc::new(p), ProviderType::LocalFs);
            }
            Err(e) => {
                tracing::error!("Failed to mount local filesystem: {}", e);
            }
        }
    }

    fn switch_to_s3(&mut self, config: S3Config) {
        match OpenDalProvider::new_s3(config.clone()) {
            Ok(p) => {
                self.config.s3 = config;
                self.switch_provider(Arc::new(p), ProviderType::S3);
            }
            Err(e) => {
                tracing::error!("Failed to mount S3 bucket: {}", e);
            }
        }
    }

    fn switch_to_subsonic(&mut self, config: SubsonicConfig) {
        let p = SubsonicProvider::new(config.clone());
        self.config.subsonic = config;
        self.switch_provider(Arc::new(p), ProviderType::Subsonic);
    }

    fn switch_to_spotify(&mut self, config: SpotifyConfig) {
        let p = SpotifyProvider::new(config.clone());
        self.config.spotify = config;
        self.switch_provider(Arc::new(p), ProviderType::Spotify);
    }

    fn switch_to_demo(&mut self) {
        self.switch_provider(Arc::new(DemoProvider::new()), ProviderType::Demo);
    }
}

impl eframe::App for SpotyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                PlayerEvent::StateChanged(state) => {
                    self.controller.update_cached_state(state);
                    ctx.request_repaint();
                }
                PlayerEvent::VisualizerLevels(levels) => {
                    self.visualizer_levels = levels;
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
        let is_playing = state.status == PlaybackStatus::Playing;

        // Top Panel: Navigation & Search
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let back_enabled = self.nav.can_go_back();
                if ui.add_enabled(back_enabled, egui::Button::new("◀")).on_hover_text("Go back").clicked() {
                    self.nav.back();
                }

                let fwd_enabled = self.nav.can_go_forward();
                if ui.add_enabled(fwd_enabled, egui::Button::new("▶")).on_hover_text("Go forward").clicked() {
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
                // Left: Track cover art & details
                ui.allocate_ui(egui::vec2(260.0, 48.0), |ui| {
                    ui.horizontal(|ui| {
                        if let Some(track) = &state.current_track {
                            if let Some(uri) = &track.cover_uri {
                                ui.add(
                                    egui::Image::new(uri)
                                        .fit_to_exact_size(egui::vec2(44.0, 44.0))
                                        .corner_radius(egui::CornerRadius::same(4)),
                                );
                            } else {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, egui::CornerRadius::same(4), egui::Color32::from_rgb(0x24, 0x24, 0x24));
                                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "🎵", egui::FontId::proportional(20.0), egui::Color32::WHITE);
                            }

                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&track.title).strong().color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new(&track.artist).small().color(egui::Color32::GRAY));
                            });
                        } else {
                            ui.label(egui::RichText::new("No track playing").weak());
                        }
                    });
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

                        if ui.button(egui::RichText::new("🔀").color(shuffle_btn_color)).on_hover_text("Shuffle").clicked() {
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

                // Right: Volume control & Audio Spectrum Visualizer
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut volume = state.volume;
                    if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0).show_value(false)).changed() {
                        self.controller.set_volume(volume);
                    }

                    let mute_icon = if state.is_muted { "🔇" } else { "🔊" };
                    if ui.button(mute_icon).clicked() {
                        self.controller.toggle_mute();
                    }

                    ui.add_space(16.0);

                    // Real-time audio spectrum visualizer
                    VisualizerWidget::new(&self.visualizer_levels, is_playing).show(ui);
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

        // Central Panel
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
                    let mut switch_demo = false;
                    let mut switch_fs = None;
                    let mut switch_s3 = None;
                    let mut switch_subsonic = None;
                    let mut switch_spotify = None;

                    SettingsView::show(
                        ui,
                        &provider_name,
                        &mut self.config,
                        &mut || switch_demo = true,
                        &mut |path| switch_fs = Some(path),
                        &mut |s3| switch_s3 = Some(s3),
                        &mut |subsonic| switch_subsonic = Some(subsonic),
                        &mut |spotify| switch_spotify = Some(spotify),
                    );

                    if switch_demo {
                        self.switch_to_demo();
                    }
                    if let Some(path) = switch_fs {
                        self.switch_to_local_fs(path);
                    }
                    if let Some(s3) = switch_s3 {
                        self.switch_to_s3(s3);
                    }
                    if let Some(subsonic) = switch_subsonic {
                        self.switch_to_subsonic(subsonic);
                    }
                    if let Some(spotify) = switch_spotify {
                        self.switch_to_spotify(spotify);
                    }
                }
            }
        });
    }
}
