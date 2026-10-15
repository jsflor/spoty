use crate::navigation::Route;
use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_HIGHLIGHT, COLOR_DIVIDER, COLOR_TEXT_BASE,
    COLOR_TEXT_SUBDUED,
};
use crate::views::{ViewAction, ViewContext};
use crate::widgets::render_track_context_menu;
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use spoty_core::{AlbumId, ArtistId};

pub struct SearchView;

impl SearchView {
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &ViewContext<'_>,
        search_query: &mut String,
    ) -> Option<ViewAction> {
        let mut action = None;

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
            let matches: Vec<_> = ctx
                .library
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
                    let is_current = ctx.current_track_id == Some(&track.id);
                    let row_height = 54.0;
                    let (row_rect, row_resp) =
                        ui.allocate_exact_size(Vec2::new(table_width, row_height), Sense::click());

                    let (artist_galley, artist_rect) = {
                        let p = ui.painter();
                        let galley = p.layout_no_wrap(
                            track.artist.clone(),
                            FontId::proportional(13.0_f32),
                            COLOR_TEXT_SUBDUED,
                        );
                        let artist_x = row_rect.left() + (table_width * 0.42);
                        let pos =
                            Pos2::new(artist_x, row_rect.center().y - (galley.size().y / 2.0));
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
                        let album_x = row_rect.left() + (table_width * 0.68);
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

                        // Artist (Clickable Link)
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

                        // Album (Clickable Link)
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
                        } else {
                            let album_x = row_rect.left() + (table_width * 0.68);
                            p.text(
                                Pos2::new(album_x, row_rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                "-",
                                FontId::proportional(13.0),
                                COLOR_TEXT_SUBDUED,
                            );
                        }

                        // Heart Button
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

                    let artist_clicked = artist_resp.clicked();
                    let album_clicked = maybe_album_resp
                        .as_ref()
                        .map(|r| r.clicked())
                        .unwrap_or(false);

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
                    } else if heart_resp.clicked() {
                        action = Some(ViewAction::ToggleFavorite(track.id.clone()));
                    } else if row_resp.clicked() {
                        action = Some(ViewAction::PlayTrack((*track).clone()));
                    }

                    ui.add_space(2.0);
                }
            }
        }

        action
    }
}
