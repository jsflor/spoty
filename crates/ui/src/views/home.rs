use eframe::egui;
use spoty_core::{Playlist, Track};
use spoty_storage::Library;
use crate::navigation::Route;

pub struct HomeView;

impl HomeView {
    pub fn show(
        ui: &mut egui::Ui,
        library: &Library,
        current_track_id: Option<&spoty_core::TrackId>,
        on_navigate: &mut impl FnMut(Route),
        on_play_track: &mut impl FnMut(Track),
        on_play_playlist: &mut impl FnMut(Playlist),
    ) {
        let greeting = get_time_greeting();
        ui.heading(egui::RichText::new(greeting).size(26.0).strong().color(egui::Color32::WHITE));
        ui.add_space(16.0);

        // Spotify's quick-access recent 2-column rectangular cards
        ui.label(egui::RichText::new("Quick Play").size(16.0).strong().color(egui::Color32::WHITE));
        ui.add_space(8.0);

        egui::Grid::new("quick_play_grid")
            .spacing([12.0, 12.0])
            .min_col_width(240.0)
            .show(ui, |ui| {
                for (i, playlist) in library.playlists.iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_width(240.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📁").size(24.0));
                            ui.vertical(|ui| {
                                if ui.link(egui::RichText::new(&playlist.title).strong().color(egui::Color32::WHITE)).clicked() {
                                    on_navigate(Route::Playlist(playlist.id.clone()));
                                }
                                ui.label(egui::RichText::new(format!("{} tracks", playlist.track_ids.len())).small().color(egui::Color32::GRAY));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("▶").color(egui::Color32::from_rgb(0x1e, 0xd7, 0x60))).clicked() {
                                    on_play_playlist(playlist.clone());
                                }
                            });
                        });
                    });

                    if (i + 1) % 2 == 0 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(24.0);

        // Featured Tracks section
        ui.label(egui::RichText::new("Featured Tracks").size(18.0).strong().color(egui::Color32::WHITE));
        ui.add_space(10.0);

        ui.horizontal_wrapped(|ui| {
            for track in &library.tracks {
                let is_current = current_track_id == Some(&track.id);

                ui.group(|ui| {
                    ui.set_width(170.0);
                    ui.set_height(200.0);
                    ui.vertical(|ui| {
                        // Card placeholder art
                        let (rect, _response) = ui.allocate_exact_size(egui::vec2(150.0, 120.0), egui::Sense::hover());
                        ui.painter().rect_filled(
                            rect,
                            egui::CornerRadius::same(6),
                            egui::Color32::from_rgb(0x28, 0x28, 0x28),
                        );

                        let title_color = if is_current {
                            egui::Color32::from_rgb(0x1e, 0xd7, 0x60)
                        } else {
                            egui::Color32::WHITE
                        };

                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(&track.title).strong().color(title_color));
                        ui.label(egui::RichText::new(&track.artist).small().color(egui::Color32::GRAY));

                        if ui.button(if is_current { "⏸" } else { "▶ Play" }).clicked() {
                            on_play_track(track.clone());
                        }
                    });
                });
                ui.add_space(12.0);
            }
        });
    }
}

fn get_time_greeting() -> &'static str {
    // Simple greeting fallback
    "Good evening"
}
