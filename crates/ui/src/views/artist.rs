use crate::navigation::Route;
use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_HIGHLIGHT, COLOR_TEXT_BASE, COLOR_TEXT_SUBDUED,
};
use crate::views::{ViewAction, ViewContext};
use crate::widgets::{play_circle_button, render_track_context_menu};
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use spoty_core::{Album, AlbumId, ArtistId, Track};

pub struct ArtistView;

impl ArtistView {
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &ViewContext<'_>,
        artist_id: &ArtistId,
    ) -> Option<ViewAction> {
        let mut action = None;

        let artist = match ctx.library.artists.iter().find(|a| &a.id == artist_id) {
            Some(a) => a,
            None => {
                ui.label(egui::RichText::new("Artist not found").color(COLOR_TEXT_SUBDUED));
                return None;
            }
        };

        // Gather all tracks belonging to this artist
        let artist_tracks: Vec<Track> = ctx
            .library
            .tracks
            .iter()
            .filter(|t| t.artist.eq_ignore_ascii_case(&artist.name))
            .cloned()
            .collect();

        // Gather all albums belonging to this artist
        let artist_albums: Vec<Album> = ctx
            .library
            .albums
            .iter()
            .filter(|a| a.artist.eq_ignore_ascii_case(&artist.name))
            .cloned()
            .collect();

        // 1. Spotify Full-Bleed Hero Portrait Banner
        egui::Frame::new()
            .fill(Color32::from_rgb(0x24, 0x24, 0x36)) // Atmospheric deep charcoal / slate violet
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(24))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 140x140 Circular Portrait
                    let portrait_size = Vec2::splat(140.0);
                    let (portrait_rect, _) = ui.allocate_exact_size(portrait_size, Sense::hover());
                    if ui.is_rect_visible(portrait_rect) {
                        let p = ui.painter();
                        let portrait_center = portrait_rect.center();
                        let radius = 70.0;

                        let avatar_uri = artist
                            .avatar_uri
                            .as_ref()
                            .or_else(|| artist_tracks.iter().find_map(|t| t.cover_uri.as_ref()));

                        if let Some(uri) = avatar_uri {
                            let img = egui::Image::new(uri)
                                .fit_to_exact_size(portrait_size)
                                .corner_radius(CornerRadius::same(70));
                            img.paint_at(ui, portrait_rect);
                        } else {
                            p.circle_filled(portrait_center, radius, COLOR_BG_CARD);
                            p.circle_stroke(
                                portrait_center,
                                radius,
                                Stroke::new(1.0_f32, Color32::from_black_alpha(40)),
                            );
                            p.text(
                                portrait_center,
                                egui::Align2::CENTER_CENTER,
                                "👤",
                                FontId::proportional(48.0),
                                COLOR_TEXT_SUBDUED,
                            );
                        }
                    }

                    ui.add_space(20.0);

                    // Hero text details
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("✓ Verified Artist")
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::from_rgb(0x3d, 0x91, 0xf4)),
                            );
                        });
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(&artist.name)
                                .size(42.0)
                                .strong()
                                .color(COLOR_TEXT_BASE),
                        );
                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new(format!(
                                "{} songs in library",
                                artist_tracks.len()
                            ))
                            .size(13.0)
                            .color(COLOR_TEXT_SUBDUED),
                        );
                    });
                });
            });

        ui.add_space(18.0);

        // 2. Action Bar: 48px Circular Green Play Button
        if !artist_tracks.is_empty() {
            ui.horizontal(|ui| {
                if play_circle_button(ui, 48.0, false, Some(COLOR_ACCENT_GREEN))
                    .on_hover_text("Play Artist")
                    .clicked()
                {
                    action = Some(ViewAction::PlayAll(artist_tracks.clone()));
                }
            });
            ui.add_space(18.0);
        }

        // 3. "Popular Tracks" Section
        if !artist_tracks.is_empty() {
            ui.heading(
                egui::RichText::new("Popular")
                    .size(20.0)
                    .strong()
                    .color(COLOR_TEXT_BASE),
            );
            ui.add_space(10.0);

            let table_width = ui.available_width();
            let popular_tracks = artist_tracks.iter().take(5);

            for (index, track) in popular_tracks.enumerate() {
                let is_current = ctx.current_track_id == Some(&track.id);
                let row_height = 54.0;
                let (row_rect, row_resp) =
                    ui.allocate_exact_size(Vec2::new(table_width, row_height), Sense::click());

                let maybe_album_data = track.album.as_ref().map(|album_name| {
                    let p = ui.painter();
                    let galley = p.layout_no_wrap(
                        album_name.clone(),
                        FontId::proportional(13.0_f32),
                        COLOR_TEXT_SUBDUED,
                    );
                    let album_x = row_rect.left() + (table_width * 0.55);
                    let pos = Pos2::new(album_x, row_rect.center().y - (galley.size().y / 2.0));
                    let rect = Rect::from_min_size(pos, galley.size());
                    (galley, rect)
                });

                let maybe_album_resp = maybe_album_data
                    .as_ref()
                    .map(|(_, rect)| ui.allocate_rect(*rect, Sense::click()));

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

                    // Track title
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

                    // Album name (clickable link)
                    if let (Some((galley, rect)), Some(resp)) =
                        (&maybe_album_data, &maybe_album_resp)
                    {
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
                    }

                    // Heart icon
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

                let album_clicked = maybe_album_resp
                    .as_ref()
                    .map(|r| r.clicked())
                    .unwrap_or(false);

                if album_clicked {
                    if let Some(album_name) = &track.album {
                        let album_id = ctx
                            .library
                            .albums
                            .iter()
                            .find(|a| {
                                a.title.eq_ignore_ascii_case(album_name)
                                    && a.artist.eq_ignore_ascii_case(&track.artist)
                            })
                            .map(|a| a.id.clone())
                            .unwrap_or_else(|| {
                                let slug = album_name
                                    .to_lowercase()
                                    .replace(|c: char| !c.is_alphanumeric(), "_");
                                AlbumId::new(format!("alb_{slug}"))
                            });
                        action = Some(ViewAction::Navigate(Route::Album(album_id)));
                    }
                } else if heart_resp.clicked() {
                    action = Some(ViewAction::ToggleFavorite(track.id.clone()));
                } else if row_resp.clicked() {
                    action = Some(ViewAction::PlayTrack(track.clone()));
                }

                ui.add_space(2.0);
            }
        }

        // 4. "Albums & Singles" Discography Section
        if !artist_albums.is_empty() {
            ui.add_space(24.0);
            ui.heading(
                egui::RichText::new("Albums & Singles")
                    .size(20.0)
                    .strong()
                    .color(COLOR_TEXT_BASE),
            );
            ui.add_space(14.0);

            let disc_scroll_id = ui.make_persistent_id("artist_discography_carousel");
            let offset = ui.data_mut(|d| d.get_temp::<f32>(disc_scroll_id).unwrap_or(0.0));

            let disc_output = egui::ScrollArea::horizontal()
                .id_salt(disc_scroll_id)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .horizontal_scroll_offset(offset)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let card_size = Vec2::new(170.0, 230.0);

                        for album in &artist_albums {
                            let (rect, resp) = ui.allocate_exact_size(card_size, Sense::click());

                            if ui.is_rect_visible(rect) {
                                let p = ui.painter();
                                let is_hovered = resp.hovered();

                                let bg = if is_hovered {
                                    COLOR_BG_HIGHLIGHT
                                } else {
                                    COLOR_BG_CARD
                                };
                                p.rect_filled(rect, CornerRadius::same(8), bg);

                                // 146x146 Artwork
                                let cover_rect = Rect::from_min_size(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 12.0),
                                    Vec2::splat(146.0),
                                );

                                let cover_uri = album.cover_uri.as_ref().or_else(|| {
                                    ctx.library.tracks.iter().find_map(|t| {
                                        if t.album.as_deref() == Some(&album.title)
                                            && t.artist.eq_ignore_ascii_case(&album.artist)
                                        {
                                            t.cover_uri.as_ref()
                                        } else {
                                            None
                                        }
                                    })
                                });

                                if let Some(uri) = cover_uri {
                                    let img = egui::Image::new(uri)
                                        .fit_to_exact_size(Vec2::splat(146.0))
                                        .corner_radius(CornerRadius::same(6));
                                    img.paint_at(ui, cover_rect);
                                } else {
                                    p.rect_filled(
                                        cover_rect,
                                        CornerRadius::same(6),
                                        Color32::from_rgb(0x24, 0x24, 0x24),
                                    );
                                    p.text(
                                        cover_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "🎵",
                                        FontId::proportional(38.0),
                                        COLOR_TEXT_SUBDUED,
                                    );
                                }

                                // Floating Play Button on card hover
                                if is_hovered {
                                    let btn_center =
                                        Pos2::new(cover_rect.max.x - 20.0, cover_rect.max.y - 20.0);
                                    let btn_radius = 18.0;
                                    p.circle_filled(btn_center, btn_radius, COLOR_ACCENT_GREEN);
                                    p.circle_stroke(
                                        btn_center,
                                        btn_radius,
                                        Stroke::new(1.0_f32, Color32::from_black_alpha(40)),
                                    );

                                    let p1 = Pos2::new(btn_center.x - 4.5, btn_center.y - 6.5);
                                    let p2 = Pos2::new(btn_center.x + 6.5, btn_center.y);
                                    let p3 = Pos2::new(btn_center.x - 4.5, btn_center.y + 6.5);
                                    p.add(egui::Shape::convex_polygon(
                                        vec![p1, p2, p3],
                                        Color32::BLACK,
                                        Stroke::NONE,
                                    ));
                                }

                                // Album title
                                p.text(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 168.0),
                                    egui::Align2::LEFT_TOP,
                                    &album.title,
                                    FontId::proportional(14.0),
                                    COLOR_TEXT_BASE,
                                );

                                // Year or Album label
                                let subtitle = match album.year {
                                    Some(year) => format!("{year} • Album"),
                                    None => "Album".to_string(),
                                };
                                p.text(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 190.0),
                                    egui::Align2::LEFT_TOP,
                                    subtitle,
                                    FontId::proportional(12.0),
                                    COLOR_TEXT_SUBDUED,
                                );
                            }

                            if resp.clicked() {
                                let mut played = false;
                                if let Some(pos) = resp.interact_pointer_pos() {
                                    let cover_max = Pos2::new(
                                        rect.min.x + 12.0 + 146.0,
                                        rect.min.y + 12.0 + 146.0,
                                    );
                                    let btn_center =
                                        Pos2::new(cover_max.x - 20.0, cover_max.y - 20.0);
                                    if pos.distance(btn_center) <= 20.0 {
                                        let album_tracks: Vec<Track> = album
                                            .track_ids
                                            .iter()
                                            .filter_map(|id| {
                                                ctx.library
                                                    .tracks
                                                    .iter()
                                                    .find(|t| &t.id == id)
                                                    .cloned()
                                            })
                                            .collect();
                                        if !album_tracks.is_empty() {
                                            action = Some(ViewAction::PlayAll(album_tracks));
                                            played = true;
                                        }
                                    }
                                }
                                if !played {
                                    action =
                                        Some(ViewAction::Navigate(Route::Album(album.id.clone())));
                                }
                            }

                            ui.add_space(14.0);
                        }
                    });
                });

            ui.data_mut(|d| d.insert_temp(disc_scroll_id, disc_output.state.offset.x));
        }

        ui.add_space(24.0);
        action
    }
}
