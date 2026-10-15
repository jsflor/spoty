use eframe::egui;
use spoty_audio::{PlayerController, PlayerEvent};
use spoty_core::{PlaybackStatus, Playlist, PlaylistId, Queue, Track};
use spoty_storage::{
    DemoProvider, Library, LibraryDatabase, MusicProvider, OpenDalProvider, S3Config,
    SpotifyConfig, SpotifyProvider, SubsonicConfig, SubsonicProvider,
};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tokio::io::AsyncReadExt;

use crate::config::{ProviderType, SpotyConfig};
use crate::navigation::{NavigationHistory, Route};
use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_BASE, COLOR_BG_CARD, COLOR_BG_ELEVATED, COLOR_BG_HIGHLIGHT,
    COLOR_CANVAS, COLOR_DIVIDER, COLOR_TEXT_BASE, COLOR_TEXT_SUBDUED,
};
use crate::views::{HomeView, PlaylistView, SearchView, SettingsView, SongsMode, SongsView};
use crate::widgets::{
    SpotifySlider, VisualizerWidget, heart_button, next_button, play_circle_button, prev_button,
    queue_button, repeat_button, shuffle_button, speaker_button,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarCategory {
    Playlists,
    Artists,
}

#[derive(Debug, Clone)]
pub struct PlaylistEditModal {
    pub id: PlaylistId,
    pub title: String,
    pub description: String,
}

pub struct SpotyApp {
    library: Library,
    provider: Arc<dyn MusicProvider>,
    controller: PlayerController,
    event_rx: Receiver<PlayerEvent>,
    tokio_rt: Arc<tokio::runtime::Runtime>,
    nav: NavigationHistory,
    queue: Queue,
    config: SpotyConfig,
    db: LibraryDatabase,
    search_query: String,
    visualizer_levels: [f32; 16],
    pending_delete_playlist: Option<Playlist>,
    editing_playlist: Option<PlaylistEditModal>,
    sidebar_category: Option<SidebarCategory>,
    queue_drawer_open: bool,
}

impl SpotyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tokio_rt =
            Arc::new(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));
        let config = SpotyConfig::load();

        // Initialize embedded SQLite database
        let db_path = SpotyConfig::config_path()
            .parent()
            .map(|p| p.join("library.db"))
            .unwrap_or_else(|| std::path::PathBuf::from("library.db"));

        let db = LibraryDatabase::open_or_create(&db_path)
            .unwrap_or_else(|_| LibraryDatabase::open_in_memory().unwrap());

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

        // Load library from provider, upsert tracks into SQLite
        let mut library = tokio_rt
            .block_on(provider.fetch_library())
            .unwrap_or_default();

        let _ = db.upsert_tracks(&library.tracks);

        // Merge any user-created playlists from SQLite
        if let Ok(db_lib) = db.load_library() {
            for pl in db_lib.playlists {
                if !library.playlists.iter().any(|p| p.id == pl.id) {
                    library.playlists.push(pl);
                }
            }
        }

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
            db,
            search_query: String::new(),
            visualizer_levels: [0.0; 16],
            pending_delete_playlist: None,
            editing_playlist: None,
            sidebar_category: None,
            queue_drawer_open: false,
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
            let _ = self.db.upsert_tracks(&new_lib.tracks);
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

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }

        let mut toggle_play = false;
        let mut toggle_mute = false;
        let mut toggle_shuffle = false;
        let mut cycle_repeat = false;
        let mut seek_delta: Option<f64> = None;
        let mut vol_delta: Option<f32> = None;
        let mut next_track = false;
        let mut prev_track = false;

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                toggle_play = true;
            }
            if i.key_pressed(egui::Key::M) {
                toggle_mute = true;
            }
            if i.key_pressed(egui::Key::S) {
                toggle_shuffle = true;
            }
            if i.key_pressed(egui::Key::R) {
                cycle_repeat = true;
            }

            if i.key_pressed(egui::Key::ArrowRight) {
                if i.modifiers.shift {
                    next_track = true;
                } else {
                    seek_delta = Some(5.0);
                }
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                if i.modifiers.shift {
                    prev_track = true;
                } else {
                    seek_delta = Some(-5.0);
                }
            }

            if i.key_pressed(egui::Key::ArrowUp) {
                vol_delta = Some(0.05);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                vol_delta = Some(-0.05);
            }
        });

        if toggle_play {
            self.controller.toggle_play_pause();
        }
        if toggle_mute {
            self.controller.toggle_mute();
        }
        if toggle_shuffle {
            let is_shuf = self.queue.toggle_shuffle();
            if is_shuf != self.controller.state().is_shuffle {
                self.controller.toggle_shuffle();
            }
        }
        if cycle_repeat {
            self.controller.cycle_repeat_mode();
        }
        if next_track {
            self.play_next();
        }
        if prev_track {
            self.play_prev();
        }
        if let Some(delta) = seek_delta {
            let current = self.controller.state().position_secs;
            let target = (current + delta).clamp(0.0, self.controller.state().duration_secs);
            self.controller.seek(Duration::from_secs_f64(target));
        }
        if let Some(delta) = vol_delta {
            let current = self.controller.state().volume;
            let target = (current + delta).clamp(0.0, 1.0);
            self.controller.set_volume(target);
        }
    }
}

impl eframe::App for SpotyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard_shortcuts(ctx);

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
        egui::TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::NONE
                    .fill(COLOR_CANVAS)
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Back circular pill button
                    let back_enabled = self.nav.can_go_back();
                    let back_bg = if back_enabled {
                        COLOR_BG_ELEVATED
                    } else {
                        COLOR_BG_BASE
                    };
                    let (back_rect, back_resp) =
                        ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
                    if ui.is_rect_visible(back_rect) {
                        let p = ui.painter();
                        p.circle_filled(back_rect.center(), 16.0, back_bg);
                        let col = if back_enabled {
                            COLOR_TEXT_BASE
                        } else {
                            COLOR_TEXT_SUBDUED
                        };
                        p.text(
                            back_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "‹",
                            egui::FontId::proportional(22.0),
                            col,
                        );
                    }
                    if back_enabled && back_resp.on_hover_text("Go back").clicked() {
                        self.nav.back();
                    }

                    // Forward circular pill button
                    let fwd_enabled = self.nav.can_go_forward();
                    let fwd_bg = if fwd_enabled {
                        COLOR_BG_ELEVATED
                    } else {
                        COLOR_BG_BASE
                    };
                    let (fwd_rect, fwd_resp) =
                        ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
                    if ui.is_rect_visible(fwd_rect) {
                        let p = ui.painter();
                        p.circle_filled(fwd_rect.center(), 16.0, fwd_bg);
                        let col = if fwd_enabled {
                            COLOR_TEXT_BASE
                        } else {
                            COLOR_TEXT_SUBDUED
                        };
                        p.text(
                            fwd_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "›",
                            egui::FontId::proportional(22.0),
                            col,
                        );
                    }
                    if fwd_enabled && fwd_resp.on_hover_text("Go forward").clicked() {
                        self.nav.forward();
                    }

                    ui.add_space(8.0);

                    // Home circular pill button
                    let (home_rect, home_resp) =
                        ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
                    if ui.is_rect_visible(home_rect) {
                        let p = ui.painter();
                        let is_home = *self.nav.current() == Route::Home;
                        let bg = if home_resp.hovered() || is_home {
                            COLOR_BG_HIGHLIGHT
                        } else {
                            COLOR_BG_ELEVATED
                        };
                        p.circle_filled(home_rect.center(), 18.0, bg);
                        let col = if is_home {
                            COLOR_TEXT_BASE
                        } else {
                            COLOR_TEXT_SUBDUED
                        };
                        p.text(
                            home_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "⌂",
                            egui::FontId::proportional(20.0),
                            col,
                        );
                    }
                    if home_resp.on_hover_text("Home").clicked() {
                        self.nav.push(Route::Home);
                    }

                    ui.add_space(6.0);

                    // Spotify Pill Search Box
                    egui::Frame::new()
                        .fill(COLOR_BG_ELEVATED)
                        .corner_radius(egui::CornerRadius::same(30))
                        .inner_margin(egui::Margin::symmetric(14, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("🔍")
                                        .size(13.0)
                                        .color(COLOR_TEXT_SUBDUED),
                                );
                                let search_box = egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text("What do you want to play?")
                                    .desired_width(340.0)
                                    .frame(false);

                                if ui.add(search_box).changed() {
                                    if *self.nav.current() != Route::Search {
                                        self.nav.push(Route::Search);
                                    }
                                }
                            });
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Settings pill button
                        let is_settings = *self.nav.current() == Route::Settings;
                        let (set_rect, set_resp) =
                            ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
                        if ui.is_rect_visible(set_rect) {
                            let p = ui.painter();
                            let bg = if set_resp.hovered() || is_settings {
                                COLOR_BG_HIGHLIGHT
                            } else {
                                COLOR_BG_ELEVATED
                            };
                            p.circle_filled(set_rect.center(), 18.0, bg);
                            let col = if is_settings {
                                COLOR_TEXT_BASE
                            } else {
                                COLOR_TEXT_SUBDUED
                            };
                            p.text(
                                set_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "⚙",
                                egui::FontId::proportional(17.0),
                                col,
                            );
                        }
                        if set_resp.on_hover_text("Settings & Storage").clicked() {
                            self.nav.push(Route::Settings);
                        }
                    });
                });
            });

        // Bottom Panel: Persistent Spotify Player Bar (76px height)
        egui::TopBottomPanel::bottom("player_bar")
            .frame(
                egui::Frame::NONE
                    .fill(COLOR_CANVAS)
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                ui.set_height(68.0);
                let total_width = ui.available_width();
                let side_width = (total_width * 0.28).clamp(200.0, 300.0);
                let center_width = total_width - (side_width * 2.0);

                ui.horizontal(|ui| {
                    // LEFT SECTION: Track details & favorite
                    ui.allocate_ui_with_layout(
                        egui::vec2(side_width, 56.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            if let Some(track) = &state.current_track {
                                if let Some(uri) = &track.cover_uri {
                                    ui.add(
                                        egui::Image::new(uri)
                                            .fit_to_exact_size(egui::vec2(48.0, 48.0))
                                            .corner_radius(egui::CornerRadius::same(4)),
                                    );
                                } else {
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(48.0, 48.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        egui::CornerRadius::same(4),
                                        COLOR_BG_CARD,
                                    );
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "🎵",
                                        egui::FontId::proportional(22.0),
                                        COLOR_TEXT_SUBDUED,
                                    );
                                }

                                ui.add_space(10.0);

                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&track.title)
                                            .size(13.0)
                                            .strong()
                                            .color(COLOR_TEXT_BASE),
                                    );
                                    ui.label(
                                        egui::RichText::new(&track.artist)
                                            .size(11.0)
                                            .color(COLOR_TEXT_SUBDUED),
                                    );
                                });

                                ui.add_space(8.0);

                                let is_fav = self.db.is_favorite(&track.id).unwrap_or(false);
                                if heart_button(ui, is_fav)
                                    .on_hover_text("Save to Your Library")
                                    .clicked()
                                {
                                    let _ = self.db.toggle_favorite(&track.id);
                                }
                            } else {
                                ui.label(
                                    egui::RichText::new("No track playing")
                                        .size(13.0)
                                        .color(COLOR_TEXT_SUBDUED),
                                );
                            }
                        },
                    );

                    // CENTER SECTION: Playback controls and SpotifySlider timeline
                    ui.allocate_ui_with_layout(
                        egui::vec2(center_width, 56.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            // Top: Controls
                            ui.horizontal(|ui| {
                                let is_shuf = self.queue.is_shuffle();
                                if shuffle_button(ui, is_shuf)
                                    .on_hover_text("Enable Shuffle")
                                    .clicked()
                                {
                                    let new_shuf = self.queue.toggle_shuffle();
                                    if new_shuf != state.is_shuffle {
                                        self.controller.toggle_shuffle();
                                    }
                                }

                                if prev_button(ui).on_hover_text("Previous Track").clicked() {
                                    self.play_prev();
                                }

                                if play_circle_button(ui, 34.0, is_playing, None)
                                    .on_hover_text(if is_playing { "Pause" } else { "Play" })
                                    .clicked()
                                {
                                    self.controller.toggle_play_pause();
                                }

                                if next_button(ui).on_hover_text("Next Track").clicked() {
                                    self.play_next();
                                }

                                if repeat_button(ui, state.repeat_mode)
                                    .on_hover_text("Repeat Mode")
                                    .clicked()
                                {
                                    self.controller.cycle_repeat_mode();
                                }
                            });

                            ui.add_space(2.0);

                            // Bottom: Progress bar
                            ui.horizontal(|ui| {
                                let cur_m = (state.position_secs / 60.0).floor() as u32;
                                let cur_s = (state.position_secs % 60.0).floor() as u32;
                                ui.label(
                                    egui::RichText::new(format!("{}:{:02}", cur_m, cur_s))
                                        .size(11.0)
                                        .monospace()
                                        .color(COLOR_TEXT_SUBDUED),
                                );

                                let mut pos = state.position_secs;
                                let max_duration = if state.duration_secs > 0.0 {
                                    state.duration_secs
                                } else {
                                    1.0
                                };
                                let slider_w = (center_width - 80.0).clamp(120.0, 520.0);

                                if SpotifySlider::new(&mut pos, max_duration)
                                    .width(slider_w)
                                    .tooltip_time()
                                    .show(ui)
                                    .changed()
                                {
                                    self.controller.seek(Duration::from_secs_f64(pos));
                                }

                                let tot_m = (state.duration_secs / 60.0).floor() as u32;
                                let tot_s = (state.duration_secs % 60.0).floor() as u32;
                                ui.label(
                                    egui::RichText::new(format!("{}:{:02}", tot_m, tot_s))
                                        .size(11.0)
                                        .monospace()
                                        .color(COLOR_TEXT_SUBDUED),
                                );
                            });
                        },
                    );

                    // RIGHT SECTION: Visualizer, Queue button & Volume
                    ui.allocate_ui_with_layout(
                        egui::vec2(side_width, 56.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let mut vol_f64 = state.volume as f64;
                            if SpotifySlider::new(&mut vol_f64, 1.0)
                                .width(85.0)
                                .tooltip_percent()
                                .show(ui)
                                .changed()
                            {
                                self.controller.set_volume(vol_f64 as f32);
                            }

                            if speaker_button(ui, state.is_muted, state.volume)
                                .on_hover_text("Mute / Unmute")
                                .clicked()
                            {
                                self.controller.toggle_mute();
                            }

                            ui.add_space(8.0);

                            if queue_button(ui, self.queue_drawer_open)
                                .on_hover_text(if self.queue_drawer_open {
                                    "Hide Queue"
                                } else {
                                    "Show Queue"
                                })
                                .clicked()
                            {
                                self.queue_drawer_open = !self.queue_drawer_open;
                            }

                            ui.add_space(10.0);

                            VisualizerWidget::new(&self.visualizer_levels, is_playing).show(ui);
                        },
                    );
                });
            });

        // Left Panel: "Your Library" Floating Card
        egui::SidePanel::left("library_sidebar")
            .frame(
                egui::Frame::NONE
                    .fill(COLOR_CANVAS)
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 4,
                        top: 4,
                        bottom: 4,
                    }),
            )
            .resizable(true)
            .default_width(260.0)
            .width_range(200.0..=380.0)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(COLOR_BG_BASE)
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📚").size(18.0));
                            ui.heading(
                                egui::RichText::new("Your Library")
                                    .size(16.0)
                                    .strong()
                                    .color(COLOR_TEXT_BASE),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let (btn_rect, btn_resp) = ui.allocate_exact_size(
                                        egui::vec2(28.0, 28.0),
                                        egui::Sense::click(),
                                    );
                                    if ui.is_rect_visible(btn_rect) {
                                        let p = ui.painter();
                                        let bg = if btn_resp.hovered() {
                                            COLOR_BG_HIGHLIGHT
                                        } else {
                                            COLOR_BG_BASE
                                        };
                                        p.circle_filled(btn_rect.center(), 14.0, bg);
                                        p.text(
                                            btn_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "＋",
                                            egui::FontId::proportional(16.0),
                                            COLOR_TEXT_BASE,
                                        );
                                    }
                                    if btn_resp.on_hover_text("Create playlist").clicked() {
                                        let pl_name = format!(
                                            "My Playlist #{}",
                                            self.library.playlists.len() + 1
                                        );
                                        if let Ok(new_pl) = self.db.create_playlist(&pl_name, None)
                                        {
                                            self.library.playlists.push(new_pl);
                                        }
                                    }
                                },
                            );
                        });

                        ui.add_space(8.0);

                        // Category chips (Playlists, Songs, Artists)
                        ui.horizontal(|ui| {
                            let chip =
                                |ui: &mut egui::Ui, text: &str, active: bool| -> egui::Response {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(68.0, 26.0),
                                        egui::Sense::click(),
                                    );
                                    if ui.is_rect_visible(rect) {
                                        let p = ui.painter();
                                        let (bg, text_col) = if active {
                                            (egui::Color32::WHITE, egui::Color32::BLACK)
                                        } else if resp.hovered() {
                                            (COLOR_BG_HIGHLIGHT, COLOR_TEXT_BASE)
                                        } else {
                                            (COLOR_BG_CARD, COLOR_TEXT_BASE)
                                        };
                                        p.rect_filled(rect, egui::CornerRadius::same(30), bg);
                                        p.text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            text,
                                            egui::FontId::proportional(12.0),
                                            text_col,
                                        );
                                    }
                                    resp
                                };

                            let pl_active =
                                self.sidebar_category == Some(SidebarCategory::Playlists);
                            if chip(ui, "Playlists", pl_active).clicked() {
                                self.sidebar_category = if pl_active {
                                    None
                                } else {
                                    Some(SidebarCategory::Playlists)
                                };
                            }

                            let songs_active = *self.nav.current() == Route::AllSongs;
                            if chip(ui, "Songs", songs_active).clicked() {
                                self.nav.push(Route::AllSongs);
                            }

                            let art_active =
                                self.sidebar_category == Some(SidebarCategory::Artists);
                            if chip(ui, "Artists", art_active).clicked() {
                                self.sidebar_category = if art_active {
                                    None
                                } else {
                                    Some(SidebarCategory::Artists)
                                };
                            }
                        });

                        ui.add_space(10.0);

                        let mut sidebar_delete = None;
                        let mut sidebar_rename = None;
                        let current_route = self.nav.current().clone();
                        let fav_count = self.db.get_favorites().map(|f| f.len()).unwrap_or(0);

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // Pinned: Liked Songs & All Songs
                            if self.sidebar_category.is_none()
                                || self.sidebar_category == Some(SidebarCategory::Playlists)
                            {
                                // Pinned: Liked Songs
                                let (liked_rect, liked_resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 52.0),
                                    egui::Sense::click(),
                                );
                                if ui.is_rect_visible(liked_rect) {
                                    let p = ui.painter();
                                    let is_active = current_route == Route::LikedSongs;
                                    let bg = if is_active || liked_resp.hovered() {
                                        COLOR_BG_HIGHLIGHT
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };
                                    p.rect_filled(liked_rect, egui::CornerRadius::same(6), bg);

                                    let thumb_rect = egui::Rect::from_min_size(
                                        egui::pos2(liked_rect.left() + 6.0, liked_rect.top() + 6.0),
                                        egui::vec2(40.0, 40.0),
                                    );
                                    p.rect_filled(
                                        thumb_rect,
                                        egui::CornerRadius::same(4),
                                        egui::Color32::from_rgb(0x45, 0x0a, 0xa5),
                                    );
                                    p.text(
                                        thumb_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "♥",
                                        egui::FontId::proportional(18.0),
                                        egui::Color32::WHITE,
                                    );

                                    let title_col = if is_active {
                                        COLOR_ACCENT_GREEN
                                    } else {
                                        COLOR_TEXT_BASE
                                    };
                                    p.text(
                                        egui::pos2(
                                            liked_rect.left() + 54.0,
                                            liked_rect.top() + 14.0,
                                        ),
                                        egui::Align2::LEFT_TOP,
                                        "Liked Songs",
                                        egui::FontId::proportional(13.0),
                                        title_col,
                                    );
                                    p.text(
                                        egui::pos2(
                                            liked_rect.left() + 54.0,
                                            liked_rect.top() + 30.0,
                                        ),
                                        egui::Align2::LEFT_TOP,
                                        format!("📌 Playlist • {} songs", fav_count),
                                        egui::FontId::proportional(11.0),
                                        COLOR_TEXT_SUBDUED,
                                    );
                                }
                                if liked_resp.clicked() {
                                    self.nav.push(Route::LikedSongs);
                                }

                                // Pinned: All Songs
                                let (all_rect, all_resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 52.0),
                                    egui::Sense::click(),
                                );
                                if ui.is_rect_visible(all_rect) {
                                    let p = ui.painter();
                                    let is_active = current_route == Route::AllSongs;
                                    let bg = if is_active || all_resp.hovered() {
                                        COLOR_BG_HIGHLIGHT
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };
                                    p.rect_filled(all_rect, egui::CornerRadius::same(6), bg);

                                    let thumb_rect = egui::Rect::from_min_size(
                                        egui::pos2(all_rect.left() + 6.0, all_rect.top() + 6.0),
                                        egui::vec2(40.0, 40.0),
                                    );
                                    p.rect_filled(
                                        thumb_rect,
                                        egui::CornerRadius::same(4),
                                        egui::Color32::from_rgb(0x1a, 0x38, 0x46),
                                    );
                                    p.text(
                                        thumb_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "🎵",
                                        egui::FontId::proportional(18.0),
                                        egui::Color32::WHITE,
                                    );

                                    let title_col = if is_active {
                                        COLOR_ACCENT_GREEN
                                    } else {
                                        COLOR_TEXT_BASE
                                    };
                                    p.text(
                                        egui::pos2(all_rect.left() + 54.0, all_rect.top() + 14.0),
                                        egui::Align2::LEFT_TOP,
                                        "All Songs",
                                        egui::FontId::proportional(13.0),
                                        title_col,
                                    );
                                    p.text(
                                        egui::pos2(all_rect.left() + 54.0, all_rect.top() + 30.0),
                                        egui::Align2::LEFT_TOP,
                                        format!("Tracks • {} tracks", self.library.tracks.len()),
                                        egui::FontId::proportional(11.0),
                                        COLOR_TEXT_SUBDUED,
                                    );
                                }
                                if all_resp.clicked() {
                                    self.nav.push(Route::AllSongs);
                                }

                                // Divider
                                let (div_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 8.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(div_rect.left() + 8.0, div_rect.center().y),
                                        egui::pos2(div_rect.right() - 8.0, div_rect.center().y),
                                    ],
                                    egui::Stroke::new(1.0_f32, COLOR_DIVIDER),
                                );
                            }

                            if self.sidebar_category.is_none()
                                || self.sidebar_category == Some(SidebarCategory::Playlists)
                            {
                                for playlist in &self.library.playlists {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), 52.0),
                                        egui::Sense::click(),
                                    );
                                    if ui.is_rect_visible(rect) {
                                        let p = ui.painter();
                                        let bg = if resp.hovered() {
                                            COLOR_BG_HIGHLIGHT
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        };
                                        p.rect_filled(rect, egui::CornerRadius::same(6), bg);

                                        // 40x40 thumbnail box
                                        let thumb_rect = egui::Rect::from_min_size(
                                            egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
                                            egui::vec2(40.0, 40.0),
                                        );
                                        p.rect_filled(
                                            thumb_rect,
                                            egui::CornerRadius::same(4),
                                            COLOR_BG_CARD,
                                        );
                                        p.text(
                                            thumb_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "📁",
                                            egui::FontId::proportional(18.0),
                                            COLOR_TEXT_SUBDUED,
                                        );

                                        // Text details
                                        p.text(
                                            egui::pos2(rect.left() + 54.0, rect.top() + 14.0),
                                            egui::Align2::LEFT_TOP,
                                            &playlist.title,
                                            egui::FontId::proportional(13.0),
                                            COLOR_TEXT_BASE,
                                        );
                                        p.text(
                                            egui::pos2(rect.left() + 54.0, rect.top() + 30.0),
                                            egui::Align2::LEFT_TOP,
                                            format!(
                                                "Playlist • {} tracks",
                                                playlist.track_ids.len()
                                            ),
                                            egui::FontId::proportional(11.0),
                                            COLOR_TEXT_SUBDUED,
                                        );
                                    }

                                    let is_custom = playlist.id.as_str().starts_with("custom_pl_");
                                    resp.context_menu(|ui| {
                                        ui.set_min_width(140.0);
                                        if is_custom {
                                            if ui.button("✏  Rename").clicked() {
                                                sidebar_rename = Some(PlaylistEditModal {
                                                    id: playlist.id.clone(),
                                                    title: playlist.title.clone(),
                                                    description: playlist
                                                        .description
                                                        .clone()
                                                        .unwrap_or_default(),
                                                });
                                                ui.close_menu();
                                            }
                                            if ui
                                                .button(egui::RichText::new("🗑  Delete").color(
                                                    egui::Color32::from_rgb(0xff, 0x60, 0x60),
                                                ))
                                                .clicked()
                                            {
                                                sidebar_delete = Some(playlist.clone());
                                                ui.close_menu();
                                            }
                                        } else {
                                            ui.label(
                                                egui::RichText::new("Built-in playlist")
                                                    .color(COLOR_TEXT_SUBDUED),
                                            );
                                        }
                                    });

                                    if resp.clicked() {
                                        self.nav.push(Route::Playlist(playlist.id.clone()));
                                    }
                                }
                            }

                            if (self.sidebar_category.is_none()
                                || self.sidebar_category == Some(SidebarCategory::Artists))
                                && !self.library.artists.is_empty()
                            {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new("Artists")
                                        .size(12.0)
                                        .color(COLOR_TEXT_SUBDUED),
                                );

                                for artist in &self.library.artists {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), 44.0),
                                        egui::Sense::click(),
                                    );
                                    if ui.is_rect_visible(rect) {
                                        let p = ui.painter();
                                        let bg = if resp.hovered() {
                                            COLOR_BG_HIGHLIGHT
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        };
                                        p.rect_filled(rect, egui::CornerRadius::same(6), bg);

                                        let avatar_rect = egui::Rect::from_min_size(
                                            egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
                                            egui::vec2(32.0, 32.0),
                                        );
                                        p.circle_filled(avatar_rect.center(), 16.0, COLOR_BG_CARD);
                                        p.text(
                                            avatar_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "👤",
                                            egui::FontId::proportional(15.0),
                                            COLOR_TEXT_SUBDUED,
                                        );

                                        p.text(
                                            egui::pos2(rect.left() + 48.0, rect.top() + 14.0),
                                            egui::Align2::LEFT_TOP,
                                            &artist.name,
                                            egui::FontId::proportional(13.0),
                                            COLOR_TEXT_BASE,
                                        );
                                    }
                                }
                            }
                        });

                        if let Some(pl) = sidebar_delete {
                            self.pending_delete_playlist = Some(pl);
                        }
                        if let Some(edit) = sidebar_rename {
                            self.editing_playlist = Some(edit);
                        }
                    });
            });

        // Right Panel: "Queue" Floating Card Drawer (if open)
        if self.queue_drawer_open {
            let mut jump_track = None;
            let mut remove_index = None;

            egui::SidePanel::right("queue_drawer_panel")
                .frame(
                    egui::Frame::NONE
                        .fill(COLOR_CANVAS)
                        .inner_margin(egui::Margin {
                            left: 4,
                            right: 8,
                            top: 4,
                            bottom: 4,
                        }),
                )
                .resizable(true)
                .default_width(300.0)
                .width_range(240.0..=440.0)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(COLOR_BG_BASE)
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(14))
                        .show(ui, |ui| {
                            // Header
                            ui.horizontal(|ui| {
                                ui.heading(
                                    egui::RichText::new("Queue")
                                        .size(16.0)
                                        .strong()
                                        .color(COLOR_TEXT_BASE),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        // Close button
                                        let (close_rect, close_resp) = ui.allocate_exact_size(
                                            egui::Vec2::splat(24.0),
                                            egui::Sense::click(),
                                        );
                                        if ui.is_rect_visible(close_rect) {
                                            let bg = if close_resp.hovered() {
                                                COLOR_BG_HIGHLIGHT
                                            } else {
                                                egui::Color32::TRANSPARENT
                                            };
                                            ui.painter().circle_filled(
                                                close_rect.center(),
                                                12.0,
                                                bg,
                                            );
                                            ui.painter().text(
                                                close_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                "✕",
                                                egui::FontId::proportional(12.0),
                                                COLOR_TEXT_BASE,
                                            );
                                        }
                                        if close_resp.on_hover_text("Close Queue").clicked() {
                                            self.queue_drawer_open = false;
                                        }

                                        let upcoming_empty = self.queue.upcoming_tracks().is_empty();
                                        if !upcoming_empty {
                                            ui.add_space(6.0);
                                            let clear_btn = egui::Button::new(
                                                egui::RichText::new("Clear Queue")
                                                    .size(11.0)
                                                    .color(COLOR_TEXT_SUBDUED),
                                            )
                                            .fill(COLOR_BG_CARD)
                                            .corner_radius(egui::CornerRadius::same(12));

                                            if ui
                                                .add(clear_btn)
                                                .on_hover_text("Clear all upcoming tracks")
                                                .clicked()
                                            {
                                                self.queue.clear_upcoming();
                                            }
                                        }
                                    },
                                );
                            });

                            ui.add_space(12.0);

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                // 1. Now Playing section
                                ui.label(
                                    egui::RichText::new("Now playing")
                                        .size(12.0)
                                        .strong()
                                        .color(COLOR_TEXT_SUBDUED),
                                );
                                ui.add_space(6.0);

                                if let Some(current) = self.queue.current_track() {
                                    let (row_rect, _) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), 48.0),
                                        egui::Sense::hover(),
                                    );
                                    if ui.is_rect_visible(row_rect) {
                                        let p = ui.painter();
                                        p.rect_filled(
                                            row_rect,
                                            egui::CornerRadius::same(6),
                                            COLOR_BG_HIGHLIGHT,
                                        );

                                        // Cover
                                        let thumb_rect = egui::Rect::from_min_size(
                                            egui::pos2(row_rect.left() + 4.0, row_rect.min.y + 4.0),
                                            egui::Vec2::splat(40.0),
                                        );
                                        if let Some(uri) = &current.cover_uri {
                                            let img = egui::Image::new(uri)
                                                .fit_to_exact_size(egui::Vec2::splat(40.0))
                                                .corner_radius(egui::CornerRadius::same(4));
                                            img.paint_at(ui, thumb_rect);
                                        } else {
                                            p.rect_filled(
                                                thumb_rect,
                                                egui::CornerRadius::same(4),
                                                COLOR_BG_CARD,
                                            );
                                            p.text(
                                                thumb_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                "🎵",
                                                egui::FontId::proportional(16.0),
                                                COLOR_TEXT_SUBDUED,
                                            );
                                        }

                                        // Title & Artist
                                        p.text(
                                            egui::pos2(row_rect.left() + 50.0, row_rect.top() + 10.0),
                                            egui::Align2::LEFT_TOP,
                                            &current.title,
                                            egui::FontId::proportional(13.0),
                                            COLOR_ACCENT_GREEN,
                                        );
                                        p.text(
                                            egui::pos2(row_rect.left() + 50.0, row_rect.top() + 26.0),
                                            egui::Align2::LEFT_TOP,
                                            &current.artist,
                                            egui::FontId::proportional(11.0),
                                            COLOR_TEXT_SUBDUED,
                                        );

                                        // Note icon
                                        p.text(
                                            egui::pos2(row_rect.right() - 14.0, row_rect.center().y),
                                            egui::Align2::RIGHT_CENTER,
                                            "♫",
                                            egui::FontId::proportional(14.0),
                                            COLOR_ACCENT_GREEN,
                                        );
                                    }
                                } else {
                                    ui.label(
                                        egui::RichText::new("No track currently playing")
                                            .size(12.0)
                                            .color(COLOR_TEXT_SUBDUED),
                                    );
                                }

                                ui.add_space(16.0);

                                // 2. Next Up section
                                ui.label(
                                    egui::RichText::new("Next in queue")
                                        .size(12.0)
                                        .strong()
                                        .color(COLOR_TEXT_SUBDUED),
                                );
                                ui.add_space(6.0);

                                let upcoming = self.queue.upcoming_tracks();
                                if upcoming.is_empty() {
                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new(
                                            "Queue is empty. Right-click any track and select \"Add to Queue\"!",
                                        )
                                        .size(12.0)
                                        .color(COLOR_TEXT_SUBDUED),
                                    );
                                } else {
                                    for (q_idx, track) in upcoming {
                                        let (row_rect, row_resp) = ui.allocate_exact_size(
                                            egui::vec2(ui.available_width(), 44.0),
                                            egui::Sense::click(),
                                        );
                                        let remove_rect = egui::Rect::from_center_size(
                                            egui::pos2(
                                                row_rect.right() - 14.0,
                                                row_rect.center().y,
                                            ),
                                            egui::Vec2::splat(20.0),
                                        );
                                        let remove_resp =
                                            ui.allocate_rect(remove_rect, egui::Sense::click());

                                        if ui.is_rect_visible(row_rect) {
                                            let p = ui.painter();
                                            let is_hovered = row_resp.hovered();

                                            if is_hovered {
                                                p.rect_filled(
                                                    row_rect,
                                                    egui::CornerRadius::same(4),
                                                    COLOR_BG_HIGHLIGHT,
                                                );
                                            }

                                            // Thumbnail or play icon
                                            let thumb_rect = egui::Rect::from_min_size(
                                                egui::pos2(
                                                    row_rect.left() + 4.0,
                                                    row_rect.min.y + 4.0,
                                                ),
                                                egui::Vec2::splat(36.0),
                                            );
                                            if is_hovered {
                                                p.rect_filled(
                                                    thumb_rect,
                                                    egui::CornerRadius::same(4),
                                                    COLOR_BG_CARD,
                                                );
                                                p.text(
                                                    thumb_rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    "▶",
                                                    egui::FontId::proportional(12.0),
                                                    COLOR_TEXT_BASE,
                                                );
                                            } else if let Some(uri) = &track.cover_uri {
                                                let img = egui::Image::new(uri)
                                                    .fit_to_exact_size(egui::Vec2::splat(36.0))
                                                    .corner_radius(egui::CornerRadius::same(4));
                                                img.paint_at(ui, thumb_rect);
                                            } else {
                                                p.rect_filled(
                                                    thumb_rect,
                                                    egui::CornerRadius::same(4),
                                                    COLOR_BG_CARD,
                                                );
                                                p.text(
                                                    thumb_rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    "🎵",
                                                    egui::FontId::proportional(14.0),
                                                    COLOR_TEXT_SUBDUED,
                                                );
                                            }

                                            // Title & Artist
                                            p.text(
                                                egui::pos2(
                                                    row_rect.left() + 46.0,
                                                    row_rect.top() + 8.0,
                                                ),
                                                egui::Align2::LEFT_TOP,
                                                &track.title,
                                                egui::FontId::proportional(12.0),
                                                COLOR_TEXT_BASE,
                                            );
                                            p.text(
                                                egui::pos2(
                                                    row_rect.left() + 46.0,
                                                    row_rect.top() + 24.0,
                                                ),
                                                egui::Align2::LEFT_TOP,
                                                &track.artist,
                                                egui::FontId::proportional(10.0),
                                                COLOR_TEXT_SUBDUED,
                                            );

                                            // Duration or remove button
                                            if is_hovered || remove_resp.hovered() {
                                                let col = if remove_resp.hovered() {
                                                    egui::Color32::from_rgb(0xff, 0x60, 0x60)
                                                } else {
                                                    COLOR_TEXT_SUBDUED
                                                };
                                                p.text(
                                                    remove_rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    "✕",
                                                    egui::FontId::proportional(12.0),
                                                    col,
                                                );
                                            } else {
                                                p.text(
                                                    egui::pos2(
                                                        row_rect.right() - 6.0,
                                                        row_rect.center().y,
                                                    ),
                                                    egui::Align2::RIGHT_CENTER,
                                                    track.formatted_duration(),
                                                    egui::FontId::proportional(11.0),
                                                    COLOR_TEXT_SUBDUED,
                                                );
                                            }
                                        }

                                        if remove_resp
                                            .on_hover_text("Remove from queue")
                                            .clicked()
                                        {
                                            remove_index = Some(q_idx);
                                        }

                                        if row_resp.clicked() {
                                            jump_track = Some(track.clone());
                                        }

                                        ui.add_space(2.0);
                                    }
                                }
                            });
                        });
                });

            if let Some(track) = jump_track {
                self.play_track(track);
            }
            if let Some(idx) = remove_index {
                self.queue.remove_at(idx);
            }
        }

        // Central Panel: Floating Card housing views
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(COLOR_CANVAS)
                    .inner_margin(egui::Margin {
                        left: 4,
                        right: 8,
                        top: 4,
                        bottom: 4,
                    }),
            )
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(COLOR_BG_BASE)
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, |ui| {
                        let active_track_id = state.current_track.as_ref().map(|t| &t.id);
                        let current_route = self.nav.current().clone();

                        egui::ScrollArea::vertical().show(ui, |ui| match current_route {
                            Route::Home => {
                                let mut nav_action = None;
                                let mut play_action = None;
                                let mut playlist_action = None;
                                let db_ref = &self.db;
                                let is_fav_closure = |tid: &spoty_core::TrackId| {
                                    db_ref.is_favorite(tid).unwrap_or(false)
                                };
                                let mut toggle_fav_action = None;
                                let mut add_to_queue_action = None;
                                let mut add_to_playlist_action = None;
                                let mut create_pl_action = None;

                                HomeView::show(
                                    ui,
                                    &self.library,
                                    active_track_id,
                                    &is_fav_closure,
                                    &mut |route| nav_action = Some(route),
                                    &mut |track| play_action = Some(track),
                                    &mut |playlist| playlist_action = Some(playlist),
                                    &mut |tid| toggle_fav_action = Some(tid),
                                    &mut |track| add_to_queue_action = Some(track),
                                    &mut |pl_id, trk_id| {
                                        add_to_playlist_action = Some((pl_id, trk_id))
                                    },
                                    &mut |trk_id| create_pl_action = Some(trk_id),
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
                                        .filter_map(|id| {
                                            self.library
                                                .tracks
                                                .iter()
                                                .find(|t| &t.id == id)
                                                .cloned()
                                        })
                                        .collect();
                                    self.play_all_tracks(tracks);
                                }
                                if let Some(tid) = toggle_fav_action {
                                    let _ = self.db.toggle_favorite(&tid);
                                }
                                if let Some(track) = add_to_queue_action {
                                    self.queue.push_back(track);
                                }
                                if let Some((pl_id, trk_id)) = add_to_playlist_action {
                                    let _ = self.db.add_track_to_playlist(&pl_id, &trk_id);
                                    if let Some(pl) =
                                        self.library.playlists.iter_mut().find(|p| p.id == pl_id)
                                    {
                                        if !pl.track_ids.contains(&trk_id) {
                                            pl.track_ids.push(trk_id);
                                        }
                                    }
                                }
                                if let Some(trk_id) = create_pl_action {
                                    let pl_name = format!(
                                        "My Playlist #{}",
                                        self.library.playlists.len() + 1
                                    );
                                    if let Ok(new_pl) = self.db.create_playlist(&pl_name, None) {
                                        let _ = self.db.add_track_to_playlist(&new_pl.id, &trk_id);
                                        let mut pl = new_pl;
                                        pl.track_ids.push(trk_id);
                                        self.library.playlists.push(pl);
                                    }
                                }
                            }
                            Route::Search => {
                                let mut play_action = None;
                                let db_ref = &self.db;
                                let is_fav_closure = |tid: &spoty_core::TrackId| {
                                    db_ref.is_favorite(tid).unwrap_or(false)
                                };
                                let mut toggle_fav_action = None;
                                let mut add_to_queue_action = None;
                                let mut add_to_playlist_action = None;
                                let mut create_pl_action = None;

                                SearchView::show(
                                    ui,
                                    &self.library,
                                    &mut self.search_query,
                                    active_track_id,
                                    &is_fav_closure,
                                    &mut |tid| toggle_fav_action = Some(tid),
                                    &mut |track| play_action = Some(track),
                                    &mut |track| add_to_queue_action = Some(track),
                                    &mut |pl_id, trk_id| {
                                        add_to_playlist_action = Some((pl_id, trk_id))
                                    },
                                    &mut |trk_id| create_pl_action = Some(trk_id),
                                );

                                if let Some(tid) = toggle_fav_action {
                                    let _ = self.db.toggle_favorite(&tid);
                                }
                                if let Some(track) = play_action {
                                    self.play_track(track);
                                }
                                if let Some(track) = add_to_queue_action {
                                    self.queue.push_back(track);
                                }
                                if let Some((pl_id, trk_id)) = add_to_playlist_action {
                                    let _ = self.db.add_track_to_playlist(&pl_id, &trk_id);
                                    if let Some(pl) =
                                        self.library.playlists.iter_mut().find(|p| p.id == pl_id)
                                    {
                                        if !pl.track_ids.contains(&trk_id) {
                                            pl.track_ids.push(trk_id);
                                        }
                                    }
                                }
                                if let Some(trk_id) = create_pl_action {
                                    let pl_name = format!(
                                        "My Playlist #{}",
                                        self.library.playlists.len() + 1
                                    );
                                    if let Ok(new_pl) = self.db.create_playlist(&pl_name, None) {
                                        let _ = self.db.add_track_to_playlist(&new_pl.id, &trk_id);
                                        let mut pl = new_pl;
                                        pl.track_ids.push(trk_id);
                                        self.library.playlists.push(pl);
                                    }
                                }
                            }
                            Route::AllSongs => {
                                let mut play_action = None;
                                let mut play_all_action = None;
                                let db_ref = &self.db;
                                let is_fav_closure = |tid: &spoty_core::TrackId| {
                                    db_ref.is_favorite(tid).unwrap_or(false)
                                };
                                let mut toggle_fav_action = None;
                                let mut add_to_queue_action = None;
                                let mut add_to_playlist_action = None;
                                let mut create_pl_action = None;

                                SongsView::show(
                                    ui,
                                    &self.library,
                                    SongsMode::All,
                                    active_track_id,
                                    &is_fav_closure,
                                    &mut |tid| toggle_fav_action = Some(tid),
                                    &mut |track| play_action = Some(track),
                                    &mut |tracks| play_all_action = Some(tracks),
                                    &mut |track| add_to_queue_action = Some(track),
                                    &mut |pl_id, trk_id| {
                                        add_to_playlist_action = Some((pl_id, trk_id))
                                    },
                                    &mut |trk_id| create_pl_action = Some(trk_id),
                                );

                                if let Some(tid) = toggle_fav_action {
                                    let _ = self.db.toggle_favorite(&tid);
                                }
                                if let Some(track) = play_action {
                                    self.play_track(track);
                                }
                                if let Some(tracks) = play_all_action {
                                    self.play_all_tracks(tracks);
                                }
                                if let Some(track) = add_to_queue_action {
                                    self.queue.push_back(track);
                                }
                                if let Some((pl_id, trk_id)) = add_to_playlist_action {
                                    let _ = self.db.add_track_to_playlist(&pl_id, &trk_id);
                                    if let Some(pl) =
                                        self.library.playlists.iter_mut().find(|p| p.id == pl_id)
                                    {
                                        if !pl.track_ids.contains(&trk_id) {
                                            pl.track_ids.push(trk_id);
                                        }
                                    }
                                }
                                if let Some(trk_id) = create_pl_action {
                                    let pl_name = format!(
                                        "My Playlist #{}",
                                        self.library.playlists.len() + 1
                                    );
                                    if let Ok(new_pl) = self.db.create_playlist(&pl_name, None) {
                                        let _ = self.db.add_track_to_playlist(&new_pl.id, &trk_id);
                                        let mut pl = new_pl;
                                        pl.track_ids.push(trk_id);
                                        self.library.playlists.push(pl);
                                    }
                                }
                            }
                            Route::LikedSongs => {
                                let mut play_action = None;
                                let mut play_all_action = None;
                                let db_ref = &self.db;
                                let is_fav_closure = |tid: &spoty_core::TrackId| {
                                    db_ref.is_favorite(tid).unwrap_or(false)
                                };
                                let mut toggle_fav_action = None;
                                let mut add_to_queue_action = None;
                                let mut add_to_playlist_action = None;
                                let mut create_pl_action = None;

                                SongsView::show(
                                    ui,
                                    &self.library,
                                    SongsMode::Liked,
                                    active_track_id,
                                    &is_fav_closure,
                                    &mut |tid| toggle_fav_action = Some(tid),
                                    &mut |track| play_action = Some(track),
                                    &mut |tracks| play_all_action = Some(tracks),
                                    &mut |track| add_to_queue_action = Some(track),
                                    &mut |pl_id, trk_id| {
                                        add_to_playlist_action = Some((pl_id, trk_id))
                                    },
                                    &mut |trk_id| create_pl_action = Some(trk_id),
                                );

                                if let Some(tid) = toggle_fav_action {
                                    let _ = self.db.toggle_favorite(&tid);
                                }
                                if let Some(track) = play_action {
                                    self.play_track(track);
                                }
                                if let Some(tracks) = play_all_action {
                                    self.play_all_tracks(tracks);
                                }
                                if let Some(track) = add_to_queue_action {
                                    self.queue.push_back(track);
                                }
                                if let Some((pl_id, trk_id)) = add_to_playlist_action {
                                    let _ = self.db.add_track_to_playlist(&pl_id, &trk_id);
                                    if let Some(pl) =
                                        self.library.playlists.iter_mut().find(|p| p.id == pl_id)
                                    {
                                        if !pl.track_ids.contains(&trk_id) {
                                            pl.track_ids.push(trk_id);
                                        }
                                    }
                                }
                                if let Some(trk_id) = create_pl_action {
                                    let pl_name = format!(
                                        "My Playlist #{}",
                                        self.library.playlists.len() + 1
                                    );
                                    if let Ok(new_pl) = self.db.create_playlist(&pl_name, None) {
                                        let _ = self.db.add_track_to_playlist(&new_pl.id, &trk_id);
                                        let mut pl = new_pl;
                                        pl.track_ids.push(trk_id);
                                        self.library.playlists.push(pl);
                                    }
                                }
                            }
                            Route::Playlist(playlist_id) => {
                                let mut play_action = None;
                                let mut play_all_action = None;
                                let mut edit_action = None;
                                let mut delete_action = None;
                                let mut remove_track_action = None;
                                let mut add_to_queue_action = None;
                                let mut add_to_playlist_action = None;
                                let mut create_pl_action = None;

                                let db_ref = &self.db;
                                let is_fav_closure = |tid: &spoty_core::TrackId| {
                                    db_ref.is_favorite(tid).unwrap_or(false)
                                };
                                let mut toggle_fav_action = None;

                                PlaylistView::show(
                                    ui,
                                    &self.library,
                                    &playlist_id,
                                    active_track_id,
                                    &is_fav_closure,
                                    &mut |tid| toggle_fav_action = Some(tid),
                                    &mut |track| play_action = Some(track),
                                    &mut |tracks| play_all_action = Some(tracks),
                                    &mut |pl_id| edit_action = Some(pl_id),
                                    &mut |pl_id| delete_action = Some(pl_id),
                                    &mut |pl_id, trk_id| {
                                        remove_track_action = Some((pl_id, trk_id))
                                    },
                                    &mut |track| add_to_queue_action = Some(track),
                                    &mut |pl_id, trk_id| {
                                        add_to_playlist_action = Some((pl_id, trk_id))
                                    },
                                    &mut |trk_id| create_pl_action = Some(trk_id),
                                );

                                if let Some(tid) = toggle_fav_action {
                                    let _ = self.db.toggle_favorite(&tid);
                                }
                                if let Some(track) = play_action {
                                    self.play_track(track);
                                }
                                if let Some(tracks) = play_all_action {
                                    self.play_all_tracks(tracks);
                                }
                                if let Some(pl_id) = edit_action {
                                    if let Some(pl) =
                                        self.library.playlists.iter().find(|p| p.id == pl_id)
                                    {
                                        self.editing_playlist = Some(PlaylistEditModal {
                                            id: pl.id.clone(),
                                            title: pl.title.clone(),
                                            description: pl.description.clone().unwrap_or_default(),
                                        });
                                    }
                                }
                                if let Some(pl_id) = delete_action {
                                    if let Some(pl) =
                                        self.library.playlists.iter().find(|p| p.id == pl_id)
                                    {
                                        self.pending_delete_playlist = Some(pl.clone());
                                    }
                                }
                                if let Some((pl_id, trk_id)) = remove_track_action {
                                    let _ = self.db.remove_track_from_playlist(&pl_id, &trk_id);
                                    if let Some(pl) =
                                        self.library.playlists.iter_mut().find(|p| p.id == pl_id)
                                    {
                                        pl.track_ids.retain(|id| id != &trk_id);
                                    }
                                }
                                if let Some(track) = add_to_queue_action {
                                    self.queue.push_back(track);
                                }
                                if let Some((pl_id, trk_id)) = add_to_playlist_action {
                                    let _ = self.db.add_track_to_playlist(&pl_id, &trk_id);
                                    if let Some(pl) =
                                        self.library.playlists.iter_mut().find(|p| p.id == pl_id)
                                    {
                                        if !pl.track_ids.contains(&trk_id) {
                                            pl.track_ids.push(trk_id);
                                        }
                                    }
                                }
                                if let Some(trk_id) = create_pl_action {
                                    let pl_name = format!(
                                        "My Playlist #{}",
                                        self.library.playlists.len() + 1
                                    );
                                    if let Ok(new_pl) = self.db.create_playlist(&pl_name, None) {
                                        let _ = self.db.add_track_to_playlist(&new_pl.id, &trk_id);
                                        let mut pl = new_pl;
                                        pl.track_ids.push(trk_id);
                                        self.library.playlists.push(pl);
                                    }
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
                        });
                    });
            });

        // Modal: Confirm Delete Playlist
        if let Some(pl) = &self.pending_delete_playlist.clone() {
            let mut close_modal = false;
            let mut confirm_delete = false;

            egui::Window::new("Delete from Your Library?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .fixed_size(egui::vec2(320.0, 130.0))
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(COLOR_BG_CARD)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::same(18)),
                )
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "This will delete \"{}\" from Your Library.",
                            pl.title
                        ))
                        .size(13.0)
                        .color(COLOR_TEXT_SUBDUED),
                    );
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let del_btn = egui::Button::new(
                                egui::RichText::new("Delete")
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(egui::Color32::from_rgb(0xd3, 0x2f, 0x2f))
                            .corner_radius(egui::CornerRadius::same(20));

                            if ui.add(del_btn).clicked() {
                                confirm_delete = true;
                            }

                            ui.add_space(8.0);

                            let cancel_btn = egui::Button::new(
                                egui::RichText::new("Cancel").color(COLOR_TEXT_BASE),
                            )
                            .fill(COLOR_BG_HIGHLIGHT)
                            .corner_radius(egui::CornerRadius::same(20));

                            if ui.add(cancel_btn).clicked() {
                                close_modal = true;
                            }
                        });
                    });
                });

            if confirm_delete {
                let pl_id = pl.id.clone();
                let _ = self.db.delete_playlist(&pl_id);
                self.library.playlists.retain(|p| p.id != pl_id);
                if self.nav.current() == &Route::Playlist(pl_id) {
                    self.nav.push(Route::Home);
                }
                self.pending_delete_playlist = None;
            } else if close_modal {
                self.pending_delete_playlist = None;
            }
        }

        // Modal: Edit Playlist Details
        if let Some(mut edit) = self.editing_playlist.clone() {
            let mut save = false;
            let mut cancel = false;

            egui::Window::new("Edit details")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .fixed_size(egui::vec2(360.0, 180.0))
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(COLOR_BG_CARD)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::same(18)),
                )
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Name")
                            .size(12.0)
                            .color(COLOR_TEXT_SUBDUED),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut edit.title).desired_width(f32::INFINITY),
                    );
                    ui.add_space(10.0);

                    ui.label(
                        egui::RichText::new("Description")
                            .size(12.0)
                            .color(COLOR_TEXT_SUBDUED),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut edit.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(18.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let save_btn = egui::Button::new(
                                egui::RichText::new("Save")
                                    .color(egui::Color32::BLACK)
                                    .strong(),
                            )
                            .fill(COLOR_ACCENT_GREEN)
                            .corner_radius(egui::CornerRadius::same(20));

                            if ui.add(save_btn).clicked() {
                                save = true;
                            }

                            ui.add_space(8.0);

                            let cancel_btn = egui::Button::new(
                                egui::RichText::new("Cancel").color(COLOR_TEXT_BASE),
                            )
                            .fill(COLOR_BG_HIGHLIGHT)
                            .corner_radius(egui::CornerRadius::same(20));

                            if ui.add(cancel_btn).clicked() {
                                cancel = true;
                            }
                        });
                    });
                });

            if cancel {
                self.editing_playlist = None;
            } else if save {
                let desc = if edit.description.trim().is_empty() {
                    None
                } else {
                    Some(edit.description.trim().to_string())
                };
                let _ = self
                    .db
                    .rename_playlist(&edit.id, edit.title.trim(), desc.as_deref());
                if let Some(pl) = self.library.playlists.iter_mut().find(|p| p.id == edit.id) {
                    pl.title = edit.title.trim().to_string();
                    pl.description = desc;
                }
                self.editing_playlist = None;
            } else {
                self.editing_playlist = Some(edit);
            }
        }
    }
}
