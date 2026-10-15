use crate::navigation::Route;
use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_HIGHLIGHT, COLOR_DIVIDER, COLOR_TEXT_BASE,
    COLOR_TEXT_SUBDUED,
};
use crate::views::{ViewAction, ViewContext};
use crate::widgets::{draw_close_icon, play_circle_button, render_track_context_menu};
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use spoty_core::{AlbumId, ArtistId, PlaylistId, Track};

pub struct PlaylistView;

impl PlaylistView {
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &ViewContext<'_>,
        playlist_id: &PlaylistId,
    ) -> Option<ViewAction> {
        let mut action = None;

        let playlist = match ctx.library.playlists.iter().find(|p| &p.id == playlist_id) {
            Some(p) => p,
            None => {
                ui.label(egui::RichText::new("Playlist not found").color(COLOR_TEXT_SUBDUED));
                return None;
            }
        };

        // Resolve tracks in this playlist
        let tracks: Vec<Track> = playlist
            .track_ids
            .iter()
            .filter_map(|id| ctx.library.tracks.iter().find(|t| &t.id == id).cloned())
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
                action = Some(ViewAction::PlayAll(tracks.clone()));
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
                            action = Some(ViewAction::EditPlaylist(playlist.id.clone()));
                            ui.close_menu();
                        }
                        if ui
                            .button(
                                egui::RichText::new("🗑  Delete playlist")
                                    .color(Color32::from_rgb(0xff, 0x60, 0x60)),
                            )
                            .clicked()
                        {
                            action = Some(ViewAction::DeletePlaylist(playlist.id.clone()));
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
                let pos = Pos2::new(row_rect.left() + 86.0, row_rect.min.y + 32.0);
                let rect = Rect::from_min_size(pos, galley.size());
                (galley, rect)
            };
            let artist_resp = ui.allocate_rect(artist_rect, Sense::click());

            let maybe_album_data = track.album.as_ref().map(|album_name| {
                let p = ui.painter();
                let galley = p.layout_no_wrap(
                    album_name.clone(),
                    FontId::proportional(13.0_f32),
                    COLOR_TEXT_SUBDUED,
                );
                let album_x = row_rect.left() + (table_width * 0.52);
                let pos = Pos2::new(album_x, row_rect.center().y - (galley.size().y / 2.0));
                let rect = Rect::from_min_size(pos, galley.size());
                (galley, rect)
            });
            let maybe_album_resp = maybe_album_data
                .as_ref()
                .map(|(_, rect)| ui.allocate_rect(*rect, Sense::click()));

            let remove_rect = Rect::from_center_size(
                Pos2::new(row_rect.right() - 104.0, row_rect.center().y),
                Vec2::splat(22.0),
            );
            let remove_resp = if is_custom {
                Some(ui.allocate_rect(remove_rect, Sense::click()))
            } else {
                None
            };

            let heart_rect = Rect::from_center_size(
                Pos2::new(row_rect.right() - 74.0, row_rect.center().y),
                Vec2::splat(24.0),
            );
            let heart_resp = ui.allocate_rect(heart_rect, Sense::click());

            let fav = (ctx.is_favorite)(&track.id);

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

                // Title
                let title_col = if is_current {
                    COLOR_ACCENT_GREEN
                } else {
                    COLOR_TEXT_BASE
                };
                p.text(
                    Pos2::new(row_rect.left() + 86.0, row_rect.min.y + 14.0),
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

                // Clickable Album
                if let (Some((galley, rect)), Some(resp)) = (&maybe_album_data, &maybe_album_resp) {
                    let album_hovered = resp.hovered();
                    let album_col = if album_hovered {
                        COLOR_TEXT_BASE
                    } else {
                        COLOR_TEXT_SUBDUED
                    };
                    p.galley(rect.min, galley.clone(), album_col);
                    if album_hovered {
                        p.line_segment(
                            [
                                Pos2::new(rect.left(), rect.bottom()),
                                Pos2::new(rect.right(), rect.bottom()),
                            ],
                            Stroke::new(1.0_f32, album_col),
                        );
                    }
                } else {
                    let album_x = row_rect.left() + (table_width * 0.52);
                    p.text(
                        Pos2::new(album_x, row_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "-",
                        FontId::proportional(13.0),
                        COLOR_TEXT_SUBDUED,
                    );
                }

                // Remove button for custom playlists
                if let Some(r_resp) = &remove_resp {
                    if is_hovered || r_resp.hovered() {
                        let col = if r_resp.hovered() {
                            Color32::from_rgb(0xff, 0x60, 0x60)
                        } else {
                            COLOR_TEXT_SUBDUED
                        };
                        draw_close_icon(p, remove_rect.center(), 9.0, 1.4, col);
                    }
                }

                // Interactive Heart Button on the right
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
                if let Some(act) = render_track_context_menu(
                    ui,
                    track,
                    &ctx.library.playlists,
                    Some(playlist_id),
                    fav,
                ) {
                    action = Some(act);
                }
            });

            let artist_clicked = artist_resp.clicked();
            let album_clicked = maybe_album_resp
                .as_ref()
                .map(|r| r.clicked())
                .unwrap_or(false);
            let remove_clicked = remove_resp.as_ref().map(|r| r.clicked()).unwrap_or(false);

            if artist_clicked {
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
            } else if album_clicked {
                if let Some(alb_name) = &track.album {
                    let album_id = ctx
                        .library
                        .albums
                        .iter()
                        .find(|a| {
                            a.title.eq_ignore_ascii_case(alb_name)
                                && a.artist.eq_ignore_ascii_case(&track.artist)
                        })
                        .map(|a| a.id.clone())
                        .unwrap_or_else(|| {
                            let slug = alb_name
                                .to_lowercase()
                                .replace(|c: char| !c.is_alphanumeric(), "_");
                            AlbumId::new(format!("alb_{slug}"))
                        });
                    action = Some(ViewAction::Navigate(Route::Album(album_id)));
                }
            } else if remove_clicked {
                action = Some(ViewAction::RemoveTrackFromPlaylist(
                    playlist.id.clone(),
                    track.id.clone(),
                ));
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
