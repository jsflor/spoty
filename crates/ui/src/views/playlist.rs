use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_HIGHLIGHT, COLOR_DIVIDER, COLOR_TEXT_BASE,
    COLOR_TEXT_SUBDUED,
};
use crate::widgets::{
    TrackMenuActions, draw_close_icon, play_circle_button, render_track_context_menu,
};
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use spoty_core::{PlaylistId, Track, TrackId};
use spoty_storage::Library;

pub struct PlaylistView;

impl PlaylistView {
    pub fn show(
        ui: &mut egui::Ui,
        library: &Library,
        playlist_id: &PlaylistId,
        current_track_id: Option<&spoty_core::TrackId>,
        is_track_fav: &impl Fn(&spoty_core::TrackId) -> bool,
        on_toggle_fav: &mut impl FnMut(spoty_core::TrackId),
        on_play_track: &mut impl FnMut(Track),
        on_play_all: &mut impl FnMut(Vec<Track>),
        on_edit_details: &mut impl FnMut(PlaylistId),
        on_delete_playlist: &mut impl FnMut(PlaylistId),
        on_remove_track: &mut impl FnMut(PlaylistId, TrackId),
        on_add_to_queue: &mut impl FnMut(Track),
        on_add_to_playlist: &mut impl FnMut(PlaylistId, TrackId),
        on_create_playlist_with_track: &mut impl FnMut(TrackId),
    ) {
        let playlist = match library.playlists.iter().find(|p| &p.id == playlist_id) {
            Some(p) => p,
            None => {
                ui.label(egui::RichText::new("Playlist not found").color(COLOR_TEXT_SUBDUED));
                return;
            }
        };

        // Resolve tracks in this playlist
        let tracks: Vec<Track> = playlist
            .track_ids
            .iter()
            .filter_map(|id| library.tracks.iter().find(|t| &t.id == id).cloned())
            .collect();

        let total_duration_secs: u32 = tracks.iter().map(|t| t.duration_secs).sum();
        let total_mins = total_duration_secs / 60;

        // 1. Spotify Full-Bleed Gradient Hero Banner
        egui::Frame::new()
            .fill(Color32::from_rgb(0x1a, 0x3d, 0x2e)) // Forest emerald gradient tone
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(24))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 160x160 Large Artwork Box
                    let art_size = Vec2::splat(160.0);
                    let (art_rect, _) = ui.allocate_exact_size(art_size, Sense::hover());
                    if ui.is_rect_visible(art_rect) {
                        let p = ui.painter();
                        if let Some(uri) = &playlist.cover_uri {
                            let img = egui::Image::new(uri)
                                .fit_to_exact_size(art_size)
                                .corner_radius(CornerRadius::same(4));
                            img.paint_at(ui, art_rect);
                        } else {
                            p.rect_filled(art_rect, CornerRadius::same(4), COLOR_BG_CARD);
                            p.circle_stroke(
                                art_rect.center(),
                                40.0,
                                Stroke::new(1.0_f32, Color32::from_black_alpha(40)),
                            );
                            p.text(
                                art_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🎵",
                                FontId::proportional(54.0),
                                COLOR_TEXT_SUBDUED,
                            );
                        }
                    }

                    ui.add_space(20.0);

                    // Hero text details
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("PUBLIC PLAYLIST")
                                .size(11.0)
                                .strong()
                                .color(COLOR_TEXT_BASE),
                        );
                        ui.add_space(2.0);

                        ui.label(
                            egui::RichText::new(&playlist.title)
                                .size(38.0)
                                .strong()
                                .color(COLOR_TEXT_BASE),
                        );

                        if let Some(desc) = &playlist.description {
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(desc)
                                    .size(13.0)
                                    .color(COLOR_TEXT_SUBDUED),
                            );
                        }

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Spoty")
                                    .size(12.0)
                                    .strong()
                                    .color(COLOR_TEXT_BASE),
                            );
                            ui.label(
                                egui::RichText::new("•")
                                    .size(12.0)
                                    .color(COLOR_TEXT_SUBDUED),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} songs, approx {} min",
                                    tracks.len(),
                                    total_mins
                                ))
                                .size(12.0)
                                .color(COLOR_TEXT_SUBDUED),
                            );
                        });
                    });
                });
            });

        ui.add_space(18.0);

        // 2. Action Bar: 48px Circular Green Play Button + Options
        let is_custom = playlist.id.as_str().starts_with("custom_pl_");
        ui.horizontal(|ui| {
            if play_circle_button(ui, 48.0, false, Some(COLOR_ACCENT_GREEN))
                .on_hover_text("Play Playlist")
                .clicked()
            {
                on_play_all(tracks.clone());
            }

            ui.add_space(16.0);

            ui.menu_button(
                egui::RichText::new("•••")
                    .size(18.0)
                    .color(COLOR_TEXT_SUBDUED),
                |ui| {
                    ui.set_min_width(140.0);
                    if is_custom {
                        if ui.button("✏  Edit details").clicked() {
                            on_edit_details(playlist.id.clone());
                            ui.close_menu();
                        }
                        if ui
                            .button(
                                egui::RichText::new("🗑  Delete playlist")
                                    .color(Color32::from_rgb(0xff, 0x60, 0x60)),
                            )
                            .clicked()
                        {
                            on_delete_playlist(playlist.id.clone());
                            ui.close_menu();
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("Built-in playlist").color(COLOR_TEXT_SUBDUED),
                        );
                    }
                },
            );
        });

        ui.add_space(18.0);

        // 3. Track List Header
        let table_width = ui.available_width();
        let (hdr_rect, _) = ui.allocate_exact_size(Vec2::new(table_width, 28.0), Sense::hover());
        if ui.is_rect_visible(hdr_rect) {
            let p = ui.painter();
            // #
            p.text(
                Pos2::new(hdr_rect.left() + 16.0, hdr_rect.center().y),
                egui::Align2::CENTER_CENTER,
                "#",
                FontId::proportional(12.0),
                COLOR_TEXT_SUBDUED,
            );
            // Title
            p.text(
                Pos2::new(hdr_rect.left() + 48.0, hdr_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "Title",
                FontId::proportional(12.0),
                COLOR_TEXT_SUBDUED,
            );
            // Album
            let album_x = hdr_rect.left() + (table_width * 0.52);
            p.text(
                Pos2::new(album_x, hdr_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "Album",
                FontId::proportional(12.0),
                COLOR_TEXT_SUBDUED,
            );
            // Clock icon for Duration
            p.text(
                Pos2::new(hdr_rect.right() - 36.0, hdr_rect.center().y),
                egui::Align2::CENTER_CENTER,
                "🕒",
                FontId::proportional(11.0),
                COLOR_TEXT_SUBDUED,
            );

            // Divider line
            p.line_segment(
                [
                    Pos2::new(hdr_rect.left(), hdr_rect.bottom()),
                    Pos2::new(hdr_rect.right(), hdr_rect.bottom()),
                ],
                Stroke::new(1.0_f32, COLOR_DIVIDER),
            );
        }

        ui.add_space(6.0);

        // 4. Interactive Track List Rows
        for (index, track) in tracks.iter().enumerate() {
            let is_current = current_track_id == Some(&track.id);
            let row_height = 54.0;
            let (row_rect, row_resp) =
                ui.allocate_exact_size(Vec2::new(table_width, row_height), Sense::click());

            if ui.is_rect_visible(row_rect) {
                let p = ui.painter();
                let is_hovered = row_resp.hovered();

                // Hover row background highlight
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

                // Artwork thumbnail (40x40)
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

                // Title & Artist
                let title_col = if is_current {
                    COLOR_ACCENT_GREEN
                } else {
                    COLOR_TEXT_BASE
                };
                p.text(
                    Pos2::new(row_rect.left() + 86.0, row_rect.min.y + 16.0),
                    egui::Align2::LEFT_TOP,
                    &track.title,
                    FontId::proportional(14.0),
                    title_col,
                );
                p.text(
                    Pos2::new(row_rect.left() + 86.0, row_rect.min.y + 32.0),
                    egui::Align2::LEFT_TOP,
                    &track.artist,
                    FontId::proportional(12.0),
                    COLOR_TEXT_SUBDUED,
                );

                // Album
                let album_x = row_rect.left() + (table_width * 0.52);
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
                on_play_track(track.clone());
            }

            let fav = is_track_fav(&track.id);

            let mut actions = TrackMenuActions {
                is_fav: fav,
                custom_playlists: &library.playlists,
                current_playlist_id: Some(playlist_id),
                on_toggle_fav,
                on_add_to_queue,
                on_add_to_playlist,
                on_create_playlist_with_track,
                on_remove_from_playlist: Some(on_remove_track),
            };

            row_resp.context_menu(|ui| {
                render_track_context_menu(ui, track, &mut actions);
            });

            // Remove button for custom playlists
            if is_custom {
                let remove_rect = Rect::from_center_size(
                    Pos2::new(row_rect.right() - 104.0, row_rect.center().y),
                    Vec2::splat(22.0),
                );
                let remove_resp = ui.allocate_rect(remove_rect, Sense::click());
                if ui.is_rect_visible(remove_rect) && (row_resp.hovered() || remove_resp.hovered())
                {
                    let p = ui.painter();
                    let col = if remove_resp.hovered() {
                        Color32::from_rgb(0xff, 0x60, 0x60)
                    } else {
                        COLOR_TEXT_SUBDUED
                    };
                    draw_close_icon(p, remove_rect.center(), 9.0, 1.4, col);
                }
                if remove_resp
                    .on_hover_text("Remove from this playlist")
                    .clicked()
                {
                    on_remove_track(playlist.id.clone(), track.id.clone());
                }
            }

            // Interactive Heart Button on the right
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
