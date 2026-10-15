use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_HIGHLIGHT, COLOR_DIVIDER, COLOR_TEXT_BASE,
    COLOR_TEXT_SUBDUED,
};
use crate::widgets::{TrackMenuActions, render_track_context_menu};
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use spoty_core::{PlaylistId, Track, TrackId};
use spoty_storage::Library;

pub struct SearchView;

impl SearchView {
    pub fn show(
        ui: &mut egui::Ui,
        library: &Library,
        search_query: &mut String,
        current_track_id: Option<&spoty_core::TrackId>,
        is_track_fav: &impl Fn(&spoty_core::TrackId) -> bool,
        on_toggle_fav: &mut impl FnMut(TrackId),
        on_play_track: &mut impl FnMut(Track),
        on_add_to_queue: &mut impl FnMut(Track),
        on_add_to_playlist: &mut impl FnMut(PlaylistId, TrackId),
        on_create_playlist_with_track: &mut impl FnMut(TrackId),
    ) {
        ui.heading(
            egui::RichText::new("Search")
                .size(26.0)
                .strong()
                .color(COLOR_TEXT_BASE),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("What do you want to listen to?")
                    .desired_width(ui.available_width() - 40.0),
            );
        });

        ui.add_space(20.0);

        let query = search_query.trim().to_lowercase();

        if query.is_empty() {
            // Show Spotify-like "Browse All" colorful category cards
            ui.label(
                egui::RichText::new("Browse All")
                    .size(18.0)
                    .strong()
                    .color(COLOR_TEXT_BASE),
            );
            ui.add_space(12.0);

            let categories = [
                ("Podcasts", egui::Color32::from_rgb(0x00, 0x64, 0x50)),
                ("Made For You", egui::Color32::from_rgb(0x1e, 0x32, 0x64)),
                ("Charts", egui::Color32::from_rgb(0x8d, 0x67, 0xab)),
                ("New Releases", egui::Color32::from_rgb(0xe8, 0x11, 0x5b)),
                ("Discover", egui::Color32::from_rgb(0x14, 0x8a, 0x08)),
                ("Concerts", egui::Color32::from_rgb(0xbc, 0x59, 0x00)),
            ];

            egui::Grid::new("browse_categories_grid")
                .spacing([16.0, 16.0])
                .show(ui, |ui| {
                    for (i, (name, color)) in categories.iter().enumerate() {
                        let (rect, _response) =
                            ui.allocate_exact_size(egui::vec2(160.0, 95.0), egui::Sense::click());
                        ui.painter()
                            .rect_filled(rect, egui::CornerRadius::same(8), *color);
                        ui.painter().text(
                            rect.min + egui::vec2(12.0, 12.0),
                            egui::Align2::LEFT_TOP,
                            *name,
                            egui::FontId::proportional(16.0),
                            egui::Color32::WHITE,
                        );

                        if (i + 1) % 3 == 0 {
                            ui.end_row();
                        }
                    }
                });
        } else {
            // Search results
            let matches: Vec<_> = library
                .tracks
                .iter()
                .filter(|t| {
                    t.title.to_lowercase().contains(&query)
                        || t.artist.to_lowercase().contains(&query)
                        || t.album
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&query)
                })
                .collect();

            if matches.is_empty() {
                ui.label(
                    egui::RichText::new(format!("No results found for \"{}\"", query))
                        .color(COLOR_TEXT_SUBDUED),
                );
            } else {
                ui.label(
                    egui::RichText::new(format!("Results ({} tracks)", matches.len()))
                        .strong()
                        .color(COLOR_TEXT_BASE),
                );
                ui.add_space(8.0);

                let table_width = ui.available_width();
                let (hdr_rect, _) =
                    ui.allocate_exact_size(Vec2::new(table_width, 28.0), Sense::hover());
                if ui.is_rect_visible(hdr_rect) {
                    let p = ui.painter();
                    p.text(
                        Pos2::new(hdr_rect.left() + 16.0, hdr_rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        "#",
                        FontId::proportional(12.0),
                        COLOR_TEXT_SUBDUED,
                    );
                    p.text(
                        Pos2::new(hdr_rect.left() + 86.0, hdr_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Title",
                        FontId::proportional(12.0),
                        COLOR_TEXT_SUBDUED,
                    );
                    p.text(
                        Pos2::new(hdr_rect.left() + (table_width * 0.42), hdr_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Artist",
                        FontId::proportional(12.0),
                        COLOR_TEXT_SUBDUED,
                    );
                    p.text(
                        Pos2::new(hdr_rect.left() + (table_width * 0.68), hdr_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Album",
                        FontId::proportional(12.0),
                        COLOR_TEXT_SUBDUED,
                    );
                    p.text(
                        Pos2::new(hdr_rect.right() - 36.0, hdr_rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        "🕒",
                        FontId::proportional(11.0),
                        COLOR_TEXT_SUBDUED,
                    );
                    p.line_segment(
                        [
                            Pos2::new(hdr_rect.left(), hdr_rect.bottom()),
                            Pos2::new(hdr_rect.right(), hdr_rect.bottom()),
                        ],
                        Stroke::new(1.0_f32, COLOR_DIVIDER),
                    );
                }

                ui.add_space(4.0);

                for (index, track) in matches.iter().enumerate() {
                    let is_current = current_track_id == Some(&track.id);
                    let row_height = 54.0;
                    let (row_rect, row_resp) =
                        ui.allocate_exact_size(Vec2::new(table_width, row_height), Sense::click());

                    if ui.is_rect_visible(row_rect) {
                        let p = ui.painter();
                        let is_hovered = row_resp.hovered();

                        if is_hovered {
                            p.rect_filled(row_rect, CornerRadius::same(4), COLOR_BG_HIGHLIGHT);
                        }

                        // Index or Play Icon
                        let idx_pos = Pos2::new(row_rect.left() + 16.0, row_rect.center().y);
                        if is_hovered {
                            p.text(
                                idx_pos,
                                egui::Align2::CENTER_CENTER,
                                "▶",
                                FontId::proportional(12.0),
                                COLOR_TEXT_BASE,
                            );
                        } else if is_current {
                            p.text(
                                idx_pos,
                                egui::Align2::CENTER_CENTER,
                                "♫",
                                FontId::proportional(13.0),
                                COLOR_ACCENT_GREEN,
                            );
                        } else {
                            p.text(
                                idx_pos,
                                egui::Align2::CENTER_CENTER,
                                format!("{}", index + 1),
                                FontId::proportional(13.0),
                                COLOR_TEXT_SUBDUED,
                            );
                        }

                        // 40x40 Thumbnail
                        let thumb_rect = Rect::from_min_size(
                            Pos2::new(row_rect.left() + 38.0, row_rect.min.y + 7.0),
                            Vec2::splat(40.0),
                        );
                        if let Some(uri) = &track.cover_uri {
                            let img = egui::Image::new(uri)
                                .fit_to_exact_size(Vec2::splat(40.0))
                                .corner_radius(CornerRadius::same(3));
                            img.paint_at(ui, thumb_rect);
                        } else {
                            p.rect_filled(thumb_rect, CornerRadius::same(3), COLOR_BG_CARD);
                            p.text(
                                thumb_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🎵",
                                FontId::proportional(16.0),
                                COLOR_TEXT_SUBDUED,
                            );
                        }

                        // Title
                        let title_col = if is_current {
                            COLOR_ACCENT_GREEN
                        } else {
                            COLOR_TEXT_BASE
                        };
                        p.text(
                            Pos2::new(row_rect.left() + 86.0, row_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &track.title,
                            FontId::proportional(14.0),
                            title_col,
                        );

                        // Artist
                        let artist_x = row_rect.left() + (table_width * 0.42);
                        p.text(
                            Pos2::new(artist_x, row_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &track.artist,
                            FontId::proportional(13.0),
                            COLOR_TEXT_SUBDUED,
                        );

                        // Album
                        let album_x = row_rect.left() + (table_width * 0.68);
                        let album_name = track.album.as_deref().unwrap_or("-");
                        p.text(
                            Pos2::new(album_x, row_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            album_name,
                            FontId::proportional(13.0),
                            COLOR_TEXT_SUBDUED,
                        );

                        // Duration
                        p.text(
                            Pos2::new(row_rect.right() - 36.0, row_rect.center().y),
                            egui::Align2::CENTER_CENTER,
                            track.formatted_duration(),
                            FontId::proportional(12.0),
                            COLOR_TEXT_SUBDUED,
                        );
                    }

                    if row_resp.clicked() {
                        on_play_track((*track).clone());
                    }

                    let fav = is_track_fav(&track.id);

                    let mut actions = TrackMenuActions {
                        is_fav: fav,
                        custom_playlists: &library.playlists,
                        current_playlist_id: None,
                        on_toggle_fav,
                        on_add_to_queue,
                        on_add_to_playlist,
                        on_create_playlist_with_track,
                        on_remove_from_playlist: None,
                    };
                    row_resp.context_menu(|ui| {
                        render_track_context_menu(ui, track, &mut actions);
                    });

                    // Heart Button
                    let heart_rect = Rect::from_center_size(
                        Pos2::new(row_rect.right() - 74.0, row_rect.center().y),
                        Vec2::splat(24.0),
                    );
                    let heart_resp = ui.allocate_rect(heart_rect, Sense::click());
                    if ui.is_rect_visible(heart_rect) {
                        let p = ui.painter();
                        let heart_col = if fav {
                            COLOR_ACCENT_GREEN
                        } else if heart_resp.hovered() {
                            COLOR_TEXT_BASE
                        } else {
                            Color32::TRANSPARENT
                        };
                        if fav || heart_resp.hovered() || row_resp.hovered() {
                            let heart_text = if fav { "♥" } else { "♡" };
                            p.text(
                                heart_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                heart_text,
                                FontId::proportional(16.0),
                                heart_col,
                            );
                        }
                    }
                    if heart_resp.clicked() {
                        on_toggle_fav(track.id.clone());
                    }

                    ui.add_space(2.0);
                }
            }
        }
    }
}
