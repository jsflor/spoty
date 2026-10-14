use eframe::egui;
use spoty_audio::{PlayerController, PlayerEvent};
use spoty_core::{PlaybackStatus, Queue, Track};
use spoty_storage::{
    DemoProvider, Library, LibraryDatabase, MusicProvider, OpenDalProvider, S3Config, SpotifyConfig,
    SpotifyProvider, SubsonicConfig, SubsonicProvider,
};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;

use crate::config::{ProviderType, SpotyConfig};
use crate::navigation::{NavigationHistory, Route};
use crate::theme::{
    COLOR_BG_BASE, COLOR_BG_CARD, COLOR_BG_ELEVATED, COLOR_BG_HIGHLIGHT, COLOR_CANVAS,
    COLOR_TEXT_BASE, COLOR_TEXT_SUBDUED,
};
use crate::views::{HomeView, PlaylistView, SearchView, SettingsView};
use crate::widgets::{
    heart_button, next_button, play_circle_button, prev_button, repeat_button, shuffle_button,
    speaker_button, SpotifySlider, VisualizerWidget,
};

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
}

impl SpotyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tokio_rt = Arc::new(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));
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
                    let back_bg = if back_enabled { COLOR_BG_ELEVATED } else { COLOR_BG_BASE };
                    let (back_rect, back_resp) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
                    if ui.is_rect_visible(back_rect) {
                        let p = ui.painter();
                        p.circle_filled(back_rect.center(), 16.0, back_bg);
                        let col = if back_enabled { COLOR_TEXT_BASE } else { COLOR_TEXT_SUBDUED };
                        p.text(back_rect.center(), egui::Align2::CENTER_CENTER, "‹", egui::FontId::proportional(22.0), col);
                    }
                    if back_enabled && back_resp.on_hover_text("Go back").clicked() {
                        self.nav.back();
                    }

                    // Forward circular pill button
                    let fwd_enabled = self.nav.can_go_forward();
                    let fwd_bg = if fwd_enabled { COLOR_BG_ELEVATED } else { COLOR_BG_BASE };
                    let (fwd_rect, fwd_resp) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
                    if ui.is_rect_visible(fwd_rect) {
                        let p = ui.painter();
                        p.circle_filled(fwd_rect.center(), 16.0, fwd_bg);
                        let col = if fwd_enabled { COLOR_TEXT_BASE } else { COLOR_TEXT_SUBDUED };
                        p.text(fwd_rect.center(), egui::Align2::CENTER_CENTER, "›", egui::FontId::proportional(22.0), col);
                    }
                    if fwd_enabled && fwd_resp.on_hover_text("Go forward").clicked() {
                        self.nav.forward();
                    }

                    ui.add_space(8.0);

                    // Home circular pill button
                    let (home_rect, home_resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
                    if ui.is_rect_visible(home_rect) {
                        let p = ui.painter();
                        let is_home = *self.nav.current() == Route::Home;
                        let bg = if home_resp.hovered() || is_home { COLOR_BG_HIGHLIGHT } else { COLOR_BG_ELEVATED };
                        p.circle_filled(home_rect.center(), 18.0, bg);
                        let col = if is_home { COLOR_TEXT_BASE } else { COLOR_TEXT_SUBDUED };
                        p.text(home_rect.center(), egui::Align2::CENTER_CENTER, "⌂", egui::FontId::proportional(20.0), col);
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
                                ui.label(egui::RichText::new("🔍").size(13.0).color(COLOR_TEXT_SUBDUED));
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
                        let (set_rect, set_resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
                        if ui.is_rect_visible(set_rect) {
                            let p = ui.painter();
                            let bg = if set_resp.hovered() || is_settings { COLOR_BG_HIGHLIGHT } else { COLOR_BG_ELEVATED };
                            p.circle_filled(set_rect.center(), 18.0, bg);
                            let col = if is_settings { COLOR_TEXT_BASE } else { COLOR_TEXT_SUBDUED };
                            p.text(set_rect.center(), egui::Align2::CENTER_CENTER, "⚙", egui::FontId::proportional(17.0), col);
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
                                    let (rect, _) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, egui::CornerRadius::same(4), COLOR_BG_CARD);
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
                                if heart_button(ui, is_fav).on_hover_text("Save to Your Library").clicked() {
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
                                if shuffle_button(ui, is_shuf).on_hover_text("Enable Shuffle").clicked() {
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

                                if repeat_button(ui, state.repeat_mode).on_hover_text("Repeat Mode").clicked() {
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
                                let max_duration = if state.duration_secs > 0.0 { state.duration_secs } else { 1.0 };
                                let slider_w = (center_width - 80.0).clamp(120.0, 520.0);

                                if SpotifySlider::new(&mut pos, max_duration).width(slider_w).show(ui).changed() {
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

                    // RIGHT SECTION: Visualizer & Volume
                    ui.allocate_ui_with_layout(
                        egui::vec2(side_width, 56.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let mut vol_f64 = state.volume as f64;
                            if SpotifySlider::new(&mut vol_f64, 1.0).width(85.0).show(ui).changed() {
                                self.controller.set_volume(vol_f64 as f32);
                            }

                            if speaker_button(ui, state.is_muted, state.volume).on_hover_text("Mute / Unmute").clicked() {
                                self.controller.toggle_mute();
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
                    .inner_margin(egui::Margin { left: 8, right: 4, top: 4, bottom: 4 }),
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

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (btn_rect, btn_resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
                                if ui.is_rect_visible(btn_rect) {
                                    let p = ui.painter();
                                    let bg = if btn_resp.hovered() { COLOR_BG_HIGHLIGHT } else { COLOR_BG_BASE };
                                    p.circle_filled(btn_rect.center(), 14.0, bg);
                                    p.text(btn_rect.center(), egui::Align2::CENTER_CENTER, "＋", egui::FontId::proportional(16.0), COLOR_TEXT_BASE);
                                }
                                if btn_resp.on_hover_text("Create playlist").clicked() {
                                    let pl_name = format!("My Playlist #{}", self.library.playlists.len() + 1);
                                    if let Ok(new_pl) = self.db.create_playlist(&pl_name, None) {
                                        self.library.playlists.push(new_pl);
                                    }
                                }
                            });
                        });

                        ui.add_space(8.0);

                        // Category chips (Playlists, Artists)
                        ui.horizontal(|ui| {
                            let chip = |ui: &mut egui::Ui, text: &str| -> egui::Response {
                                let (rect, resp) = ui.allocate_exact_size(egui::vec2(76.0, 26.0), egui::Sense::click());
                                if ui.is_rect_visible(rect) {
                                    let p = ui.painter();
                                    let bg = if resp.hovered() { COLOR_BG_HIGHLIGHT } else { COLOR_BG_CARD };
                                    p.rect_filled(rect, egui::CornerRadius::same(30), bg);
                                    p.text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        text,
                                        egui::FontId::proportional(12.0),
                                        COLOR_TEXT_BASE,
                                    );
                                }
                                resp
                            };

                            chip(ui, "Playlists");
                            chip(ui, "Artists");
                        });

                        ui.add_space(10.0);

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for playlist in &self.library.playlists {
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 52.0),
                                    egui::Sense::click(),
                                );
                                if ui.is_rect_visible(rect) {
                                    let p = ui.painter();
                                    let bg = if resp.hovered() { COLOR_BG_HIGHLIGHT } else { egui::Color32::TRANSPARENT };
                                    p.rect_filled(rect, egui::CornerRadius::same(6), bg);

                                    // 40x40 thumbnail box
                                    let thumb_rect = egui::Rect::from_min_size(
                                        egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
                                        egui::vec2(40.0, 40.0),
                                    );
                                    p.rect_filled(thumb_rect, egui::CornerRadius::same(4), COLOR_BG_CARD);
                                    p.text(thumb_rect.center(), egui::Align2::CENTER_CENTER, "📁", egui::FontId::proportional(18.0), COLOR_TEXT_SUBDUED);

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
                                        format!("Playlist • {} tracks", playlist.track_ids.len()),
                                        egui::FontId::proportional(11.0),
                                        COLOR_TEXT_SUBDUED,
                                    );
                                }

                                if resp.clicked() {
                                    self.nav.push(Route::Playlist(playlist.id.clone()));
                                }
                            }

                            if !self.library.artists.is_empty() {
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new("Artists").size(12.0).color(COLOR_TEXT_SUBDUED));

                                for artist in &self.library.artists {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), 44.0),
                                        egui::Sense::click(),
                                    );
                                    if ui.is_rect_visible(rect) {
                                        let p = ui.painter();
                                        let bg = if resp.hovered() { COLOR_BG_HIGHLIGHT } else { egui::Color32::TRANSPARENT };
                                        p.rect_filled(rect, egui::CornerRadius::same(6), bg);

                                        let avatar_rect = egui::Rect::from_min_size(
                                            egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
                                            egui::vec2(32.0, 32.0),
                                        );
                                        p.circle_filled(avatar_rect.center(), 16.0, COLOR_BG_CARD);
                                        p.text(avatar_rect.center(), egui::Align2::CENTER_CENTER, "👤", egui::FontId::proportional(15.0), COLOR_TEXT_SUBDUED);

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
                    });
            });

        // Central Panel: Floating Card housing views
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(COLOR_CANVAS)
                    .inner_margin(egui::Margin { left: 4, right: 8, top: 4, bottom: 4 }),
            )
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(COLOR_BG_BASE)
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, |ui| {
                        let active_track_id = state.current_track.as_ref().map(|t| &t.id);
                        let current_route = self.nav.current().clone();

                        egui::ScrollArea::vertical().show(ui, |ui| {
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

                                    let db_ref = &self.db;
                                    let is_fav_closure = |tid: &spoty_core::TrackId| db_ref.is_favorite(tid).unwrap_or(false);
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
                    });
            });
    }
}
