use crate::navigation::Route;
use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_HIGHLIGHT, COLOR_DIVIDER, COLOR_TEXT_BASE,
    COLOR_TEXT_SUBDUED,
};
use crate::views::{ViewAction, ViewContext};
use crate::widgets::{play_circle_button, render_track_context_menu};
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use spoty_core::{AlbumId, ArtistId, Track};

pub struct AlbumView;

impl AlbumView {
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &ViewContext<'_>,
        album_id: &AlbumId,
    ) -> Option<ViewAction> {
        let mut action = None;

        let album = match ctx.library.albums.iter().find(|a| &a.id == album_id) {
            Some(a) => a,
            None => {
                ui.label(egui::RichText::new("Album not found").color(COLOR_TEXT_SUBDUED));
                return None;
            }
        };

        // Resolve tracks in this album
        let tracks: Vec<Track> = if !album.track_ids.is_empty() {
            album
                .track_ids
                .iter()
                .filter_map(|id| ctx.library.tracks.iter().find(|t| &t.id == id).cloned())
                .collect()
        } else {
            ctx.library
                .tracks
                .iter()
                .filter(|t| {
                    t.album.as_deref() == Some(&album.title)
                        && t.artist.eq_ignore_ascii_case(&album.artist)
                })
                .cloned()
                .collect()
        };

        let total_duration_secs: u32 = tracks.iter().map(|t| t.duration_secs).sum();
        let total_mins = total_duration_secs / 60;

        // 1. Spotify Full-Bleed Gradient Hero Banner
        egui::Frame::new()
            .fill(Color32::from_rgb(0x1e, 0x32, 0x40)) // Slate charcoal atmospheric tone
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(24))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 160x160 Large Artwork Box
                    let art_size = Vec2::splat(160.0);
                    let (art_rect, _) = ui.allocate_exact_size(art_size, Sense::hover());
                    if ui.is_rect_visible(art_rect) {
                        let p = ui.painter();
                        let cover_uri = album
                            .cover_uri
                            .as_ref()
                            .or_else(|| tracks.iter().find_map(|t| t.cover_uri.as_ref()));

                        if let Some(uri) = cover_uri {
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
                            egui::RichText::new("ALBUM")
                                .size(11.0)
                                .strong()
                                .color(COLOR_TEXT_BASE),
                        );
                        ui.add_space(2.0);

                        ui.label(
                            egui::RichText::new(&album.title)
                                .size(38.0)
                                .strong()
                                .color(COLOR_TEXT_BASE),
                        );

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            let artist_resp = ui.link(
                                egui::RichText::new(&album.artist)
                                    .size(13.0)
                                    .strong()
                                    .color(COLOR_TEXT_BASE),
                            );
                            if artist_resp.clicked() {
                                let artist_id = ctx
                                    .library
                                    .artists
                                    .iter()
                                    .find(|a| a.name.eq_ignore_ascii_case(&album.artist))
                                    .map(|a| a.id.clone())
                                    .unwrap_or_else(|| {
                                        let slug = album
                                            .artist
                                            .to_lowercase()
                                            .replace(|c: char| !c.is_alphanumeric(), "_");
                                        ArtistId::new(format!("art_{slug}"))
                                    });
                                action = Some(ViewAction::Navigate(Route::Artist(artist_id)));
                            }

                            if let Some(year) = album.year {
                                ui.label(
                                    egui::RichText::new("•")
                                        .size(12.0)
                                        .color(COLOR_TEXT_SUBDUED),
                                );
                                ui.label(
                                    egui::RichText::new(format!("{year}"))
                                        .size(12.0)
                                        .color(COLOR_TEXT_SUBDUED),
                                );
                            }

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

        // 2. Action Bar: 48px Circular Green Play Button
        ui.horizontal(|ui| {
            if play_circle_button(ui, 48.0, false, Some(COLOR_ACCENT_GREEN))
                .on_hover_text("Play Album")
                .clicked()
            {
                action = Some(ViewAction::PlayAll(tracks.clone()));
            }
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
            let is_current = ctx.current_track_id == Some(&track.id);
            let row_height = 54.0;
            let (row_rect, row_resp) =
                ui.allocate_exact_size(Vec2::new(table_width, row_height), Sense::click());

            let (artist_galley, artist_rect) = {
                let p = ui.painter();
                let galley = p.layout_no_wrap(
                    track.artist.clone(),
                    FontId::proportional(12.0_f32),
                    COLOR_TEXT_SUBDUED,
                );
                let pos = Pos2::new(row_rect.left() + 48.0, row_rect.min.y + 32.0);
                let rect = Rect::from_min_size(pos, galley.size());
                (galley, rect)
            };
            let artist_resp = ui.allocate_rect(artist_rect, Sense::click());

            let heart_rect = Rect::from_center_size(
                Pos2::new(row_rect.right() - 74.0, row_rect.center().y),
                Vec2::splat(24.0),
            );
            let heart_resp = ui.allocate_rect(heart_rect, Sense::click());

            let fav = (ctx.is_favorite)(&track.id);

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

                // Title
                let title_col = if is_current {
                    COLOR_ACCENT_GREEN
                } else {
                    COLOR_TEXT_BASE
                };
                p.text(
                    Pos2::new(row_rect.left() + 48.0, row_rect.min.y + 14.0),
                    egui::Align2::LEFT_TOP,
                    &track.title,
                    FontId::proportional(14.0),
                    title_col,
                );

                // Clickable Artist Link underneath
                let artist_hovered = artist_resp.hovered();
                let artist_col = if artist_hovered {
                    COLOR_TEXT_BASE
                } else {
                    COLOR_TEXT_SUBDUED
                };
                p.galley(artist_rect.min, artist_galley, artist_col);
                if artist_hovered {
                    p.line_segment(
                        [
                            Pos2::new(artist_rect.left(), artist_rect.bottom()),
                            Pos2::new(artist_rect.right(), artist_rect.bottom()),
                        ],
                        Stroke::new(1.0_f32, artist_col),
                    );
                }

                // Heart Icon
                let heart_col = if fav {
                    COLOR_ACCENT_GREEN
                } else if heart_resp.hovered() {
                    COLOR_TEXT_BASE
                } else {
                    Color32::TRANSPARENT
                };
                if fav || heart_resp.hovered() || is_hovered {
                    let heart_text = if fav { "♥" } else { "♡" };
                    p.text(
                        heart_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        heart_text,
                        FontId::proportional(16.0),
                        heart_col,
                    );
                }

                // Duration
                p.text(
                    Pos2::new(row_rect.right() - 36.0, row_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    track.formatted_duration(),
                    FontId::proportional(12.0),
                    COLOR_TEXT_SUBDUED,
                );
            }

            row_resp.context_menu(|ui| {
                if let Some(act) =
                    render_track_context_menu(ui, track, &ctx.library.playlists, None, fav)
                {
                    action = Some(act);
                }
            });

            if artist_resp.clicked() {
                let artist_id = ctx
                    .library
                    .artists
                    .iter()
                    .find(|a| a.name.eq_ignore_ascii_case(&track.artist))
                    .map(|a| a.id.clone())
                    .unwrap_or_else(|| {
                        let slug = track
                            .artist
                            .to_lowercase()
                            .replace(|c: char| !c.is_alphanumeric(), "_");
                        ArtistId::new(format!("art_{slug}"))
                    });
                action = Some(ViewAction::Navigate(Route::Artist(artist_id)));
            } else if heart_resp.clicked() {
                action = Some(ViewAction::ToggleFavorite(track.id.clone()));
            } else if row_resp.clicked() {
                action = Some(ViewAction::PlayTrack(track.clone()));
            }

            ui.add_space(2.0);
        }

        action
    }
}
