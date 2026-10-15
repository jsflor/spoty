use crate::navigation::Route;
use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_HIGHLIGHT, COLOR_TEXT_BASE, COLOR_TEXT_SUBDUED,
};
use crate::views::{ViewAction, ViewContext};
use crate::widgets::render_track_context_menu;
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};

pub struct HomeView;

impl HomeView {
    pub fn show(ui: &mut egui::Ui, ctx: &ViewContext<'_>) -> Option<ViewAction> {
        let mut action = None;
        let greeting = get_time_greeting();

        // Hero Greeting
        ui.add_space(4.0);
        ui.heading(
            egui::RichText::new(greeting)
                .size(30.0)
                .strong()
                .color(COLOR_TEXT_BASE),
        );
        ui.add_space(18.0);

        // Spotify's 2-column or 3-column "Quick Play" rectangular cards (capped to first 6)
        if !ctx.library.playlists.is_empty() {
            let available_w = ui.available_width();
            let num_cols = if available_w > 900.0 { 3 } else { 2 };
            let card_w =
                ((available_w - ((num_cols - 1) as f32 * 12.0)) / num_cols as f32).max(220.0);

            egui::Grid::new("quick_play_grid")
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    for (i, playlist) in ctx.library.playlists.iter().take(6).enumerate() {
                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::new(card_w, 64.0), Sense::click());

                        if ui.is_rect_visible(rect) {
                            let p = ui.painter();
                            let is_hovered = resp.hovered();

                            // Card background
                            let bg = if is_hovered {
                                Color32::from_rgba_premultiplied(0xff, 0xff, 0xff, 0x24)
                            } else {
                                Color32::from_rgba_premultiplied(0xff, 0xff, 0xff, 0x14)
                            };
                            p.rect_filled(rect, CornerRadius::same(6), bg);

                            // Left cover (64x64)
                            let cover_rect = Rect::from_min_size(rect.min, Vec2::splat(64.0));
                            if let Some(uri) = &playlist.cover_uri {
                                let img = egui::Image::new(uri)
                                    .fit_to_exact_size(Vec2::splat(64.0))
                                    .corner_radius(CornerRadius {
                                        nw: 6,
                                        sw: 6,
                                        ne: 0,
                                        se: 0,
                                    });
                                img.paint_at(ui, cover_rect);
                            } else {
                                p.rect_filled(
                                    cover_rect,
                                    CornerRadius {
                                        nw: 6,
                                        sw: 6,
                                        ne: 0,
                                        se: 0,
                                    },
                                    COLOR_BG_CARD,
                                );
                                p.text(
                                    cover_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "📁",
                                    FontId::proportional(26.0),
                                    COLOR_TEXT_SUBDUED,
                                );
                            }

                            // Title
                            let title_rect = Rect::from_min_max(
                                Pos2::new(rect.min.x + 76.0, rect.min.y + 16.0),
                                Pos2::new(rect.max.x - 52.0, rect.max.y - 16.0),
                            );
                            p.text(
                                Pos2::new(title_rect.min.x, rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                &playlist.title,
                                FontId::proportional(14.0),
                                COLOR_TEXT_BASE,
                            );

                            // Floating Play Button on card hover
                            if is_hovered {
                                let btn_center = Pos2::new(rect.max.x - 28.0, rect.center().y);
                                let btn_radius = 18.0;
                                p.circle_filled(btn_center, btn_radius, COLOR_ACCENT_GREEN);
                                p.circle_stroke(
                                    btn_center,
                                    btn_radius,
                                    Stroke::new(1.0_f32, Color32::from_black_alpha(50)),
                                );

                                // Play triangle
                                let p1 = Pos2::new(btn_center.x - 4.5, btn_center.y - 6.5);
                                let p2 = Pos2::new(btn_center.x + 6.5, btn_center.y);
                                let p3 = Pos2::new(btn_center.x - 4.5, btn_center.y + 6.5);
                                p.add(egui::Shape::convex_polygon(
                                    vec![p1, p2, p3],
                                    Color32::BLACK,
                                    Stroke::NONE,
                                ));
                            }
                        }

                        if resp.clicked() {
                            let mut played = false;
                            if let Some(pos) = resp.interact_pointer_pos() {
                                let btn_center = Pos2::new(rect.max.x - 28.0, rect.center().y);
                                if pos.distance(btn_center) <= 20.0 {
                                    let tracks: Vec<_> = playlist
                                        .track_ids
                                        .iter()
                                        .filter_map(|id| {
                                            ctx.library.tracks.iter().find(|t| &t.id == id).cloned()
                                        })
                                        .collect();
                                    action = Some(ViewAction::PlayAll(tracks));
                                    played = true;
                                }
                            }
                            if !played {
                                action = Some(ViewAction::Navigate(Route::Playlist(
                                    playlist.id.clone(),
                                )));
                            }
                        }

                        if (i + 1) % num_cols == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(28.0);
        }

        // Section 1: "Featured & Made For You" (Tracks Carousel)
        if !ctx.library.tracks.is_empty() {
            let (prev_trk, next_trk) = render_carousel_header(ui, "Featured & Made For You");
            ui.add_space(14.0);

            let scroll_id = ui.make_persistent_id("featured_tracks_carousel");
            let mut offset = ui.data_mut(|d| d.get_temp::<f32>(scroll_id).unwrap_or(0.0));
            if prev_trk {
                offset = (offset - 420.0).max(0.0);
            }
            if next_trk {
                offset += 420.0;
            }

            let output = egui::ScrollArea::horizontal()
                .id_salt(scroll_id)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .horizontal_scroll_offset(offset)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let card_size = Vec2::new(180.0, 246.0);

                        for track in &ctx.library.tracks {
                            let is_current = ctx.current_track_id == Some(&track.id);
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

                                // 156x156 Artwork
                                let cover_rect = Rect::from_min_size(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 12.0),
                                    Vec2::splat(156.0),
                                );

                                if let Some(uri) = &track.cover_uri {
                                    let img = egui::Image::new(uri)
                                        .fit_to_exact_size(Vec2::splat(156.0))
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

                                // Floating Spotify green Play Circle Button on hover
                                if is_hovered {
                                    let btn_center =
                                        Pos2::new(cover_rect.max.x - 22.0, cover_rect.max.y - 22.0);
                                    let btn_radius = 20.0;
                                    p.circle_filled(btn_center, btn_radius, COLOR_ACCENT_GREEN);
                                    p.circle_stroke(
                                        btn_center,
                                        btn_radius,
                                        Stroke::new(1.5_f32, Color32::from_black_alpha(40)),
                                    );

                                    let p1 = Pos2::new(btn_center.x - 5.0, btn_center.y - 7.0);
                                    let p2 = Pos2::new(btn_center.x + 7.0, btn_center.y);
                                    let p3 = Pos2::new(btn_center.x - 5.0, btn_center.y + 7.0);
                                    p.add(egui::Shape::convex_polygon(
                                        vec![p1, p2, p3],
                                        Color32::BLACK,
                                        Stroke::NONE,
                                    ));
                                }

                                // Track title
                                let title_color = if is_current {
                                    COLOR_ACCENT_GREEN
                                } else {
                                    COLOR_TEXT_BASE
                                };

                                p.text(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 178.0),
                                    egui::Align2::LEFT_TOP,
                                    &track.title,
                                    FontId::proportional(14.0),
                                    title_color,
                                );

                                // Artist name
                                p.text(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 198.0),
                                    egui::Align2::LEFT_TOP,
                                    &track.artist,
                                    FontId::proportional(12.0),
                                    COLOR_TEXT_SUBDUED,
                                );
                            }

                            if resp.clicked() {
                                action = Some(ViewAction::PlayTrack(track.clone()));
                            }

                            let fav = (ctx.is_favorite)(&track.id);
                            resp.context_menu(|ui| {
                                if let Some(act) = render_track_context_menu(
                                    ui,
                                    track,
                                    &ctx.library.playlists,
                                    None,
                                    fav,
                                ) {
                                    action = Some(act);
                                }
                            });

                            ui.add_space(14.0);
                        }
                    });
                });

            ui.data_mut(|d| d.insert_temp(scroll_id, output.state.offset.x));
        }

        // Section 2: "Your Playlists & Mixes" (Playlists Carousel)
        if !ctx.library.playlists.is_empty() {
            ui.add_space(28.0);
            let (prev_pl, next_pl) = render_carousel_header(ui, "Your Playlists & Mixes");
            ui.add_space(14.0);

            let pl_scroll_id = ui.make_persistent_id("playlists_carousel");
            let mut pl_offset = ui.data_mut(|d| d.get_temp::<f32>(pl_scroll_id).unwrap_or(0.0));
            if prev_pl {
                pl_offset = (pl_offset - 420.0).max(0.0);
            }
            if next_pl {
                pl_offset += 420.0;
            }

            let pl_output = egui::ScrollArea::horizontal()
                .id_salt(pl_scroll_id)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .horizontal_scroll_offset(pl_offset)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let card_size = Vec2::new(180.0, 246.0);
                        for playlist in &ctx.library.playlists {
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

                                // Artwork 156x156
                                let cover_rect = Rect::from_min_size(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 12.0),
                                    Vec2::splat(156.0),
                                );
                                if let Some(uri) = &playlist.cover_uri {
                                    let img = egui::Image::new(uri)
                                        .fit_to_exact_size(Vec2::splat(156.0))
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
                                        "📁",
                                        FontId::proportional(38.0),
                                        COLOR_TEXT_SUBDUED,
                                    );
                                }

                                // Play button on hover
                                if is_hovered {
                                    let btn_center =
                                        Pos2::new(cover_rect.max.x - 22.0, cover_rect.max.y - 22.0);
                                    let btn_radius = 20.0;
                                    p.circle_filled(btn_center, btn_radius, COLOR_ACCENT_GREEN);
                                    p.circle_stroke(
                                        btn_center,
                                        btn_radius,
                                        Stroke::new(1.5_f32, Color32::from_black_alpha(40)),
                                    );
                                    let p1 = Pos2::new(btn_center.x - 5.0, btn_center.y - 7.0);
                                    let p2 = Pos2::new(btn_center.x + 7.0, btn_center.y);
                                    let p3 = Pos2::new(btn_center.x - 5.0, btn_center.y + 7.0);
                                    p.add(egui::Shape::convex_polygon(
                                        vec![p1, p2, p3],
                                        Color32::BLACK,
                                        Stroke::NONE,
                                    ));
                                }

                                // Playlist title
                                p.text(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 178.0),
                                    egui::Align2::LEFT_TOP,
                                    &playlist.title,
                                    FontId::proportional(14.0),
                                    COLOR_TEXT_BASE,
                                );

                                // Subtitle
                                p.text(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 198.0),
                                    egui::Align2::LEFT_TOP,
                                    format!("Playlist • {} tracks", playlist.track_ids.len()),
                                    FontId::proportional(12.0),
                                    COLOR_TEXT_SUBDUED,
                                );
                            }

                            if resp.clicked() {
                                let mut played = false;
                                if let Some(pos) = resp.interact_pointer_pos() {
                                    let cover_max = Pos2::new(
                                        rect.min.x + 12.0 + 156.0,
                                        rect.min.y + 12.0 + 156.0,
                                    );
                                    let btn_center =
                                        Pos2::new(cover_max.x - 22.0, cover_max.y - 22.0);
                                    if pos.distance(btn_center) <= 22.0 {
                                        let tracks: Vec<_> = playlist
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
                                        action = Some(ViewAction::PlayAll(tracks));
                                        played = true;
                                    }
                                }
                                if !played {
                                    action = Some(ViewAction::Navigate(Route::Playlist(
                                        playlist.id.clone(),
                                    )));
                                }
                            }

                            ui.add_space(14.0);
                        }
                    });
                });

            ui.data_mut(|d| d.insert_temp(pl_scroll_id, pl_output.state.offset.x));
        }

        // Section 3: "Artists You Might Like" (Artists Carousel)
        if !ctx.library.artists.is_empty() {
            ui.add_space(28.0);
            let (prev_art, next_art) = render_carousel_header(ui, "Artists You Might Like");
            ui.add_space(14.0);

            let art_scroll_id = ui.make_persistent_id("artists_carousel");
            let mut art_offset = ui.data_mut(|d| d.get_temp::<f32>(art_scroll_id).unwrap_or(0.0));
            if prev_art {
                art_offset = (art_offset - 420.0).max(0.0);
            }
            if next_art {
                art_offset += 420.0;
            }

            let art_output = egui::ScrollArea::horizontal()
                .id_salt(art_scroll_id)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .horizontal_scroll_offset(art_offset)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let card_size = Vec2::new(160.0, 210.0);
                        for artist in &ctx.library.artists {
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

                                // Circular avatar 116px
                                let avatar_center = Pos2::new(rect.center().x, rect.min.y + 70.0);
                                let avatar_radius = 54.0;

                                if let Some(uri) = &artist.avatar_uri {
                                    let img_rect = Rect::from_center_size(
                                        avatar_center,
                                        Vec2::splat(avatar_radius * 2.0),
                                    );
                                    let img = egui::Image::new(uri)
                                        .fit_to_exact_size(Vec2::splat(avatar_radius * 2.0))
                                        .corner_radius(CornerRadius::same(avatar_radius as u8));
                                    img.paint_at(ui, img_rect);
                                } else {
                                    p.circle_filled(
                                        avatar_center,
                                        avatar_radius,
                                        Color32::from_rgb(0x24, 0x24, 0x24),
                                    );
                                    p.text(
                                        avatar_center,
                                        egui::Align2::CENTER_CENTER,
                                        "👤",
                                        FontId::proportional(36.0),
                                        COLOR_TEXT_SUBDUED,
                                    );
                                }

                                // Artist name
                                p.text(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 144.0),
                                    egui::Align2::LEFT_TOP,
                                    &artist.name,
                                    FontId::proportional(14.0),
                                    COLOR_TEXT_BASE,
                                );

                                p.text(
                                    Pos2::new(rect.min.x + 12.0, rect.min.y + 166.0),
                                    egui::Align2::LEFT_TOP,
                                    "Artist",
                                    FontId::proportional(12.0),
                                    COLOR_TEXT_SUBDUED,
                                );
                            }

                            if resp.clicked() {
                                action =
                                    Some(ViewAction::Navigate(Route::Artist(artist.id.clone())));
                            }

                            ui.add_space(14.0);
                        }
                    });
                });

            ui.data_mut(|d| d.insert_temp(art_scroll_id, art_output.state.offset.x));
        }

        ui.add_space(24.0);
        action
    }
}

fn render_carousel_header(ui: &mut egui::Ui, title: &str) -> (bool, bool) {
    let mut prev_clicked = false;
    let mut next_clicked = false;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(title)
                .size(20.0)
                .strong()
                .color(COLOR_TEXT_BASE),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (next_rect, next_resp) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
            if ui.is_rect_visible(next_rect) {
                let bg = if next_resp.hovered() {
                    COLOR_BG_HIGHLIGHT
                } else {
                    COLOR_BG_CARD
                };
                ui.painter().circle_filled(next_rect.center(), 14.0, bg);
                ui.painter().text(
                    next_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "›",
                    FontId::proportional(18.0),
                    COLOR_TEXT_BASE,
                );
            }
            if next_resp.on_hover_text("Scroll right").clicked() {
                next_clicked = true;
            }

            ui.add_space(6.0);

            let (prev_rect, prev_resp) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
            if ui.is_rect_visible(prev_rect) {
                let bg = if prev_resp.hovered() {
                    COLOR_BG_HIGHLIGHT
                } else {
                    COLOR_BG_CARD
                };
                ui.painter().circle_filled(prev_rect.center(), 14.0, bg);
                ui.painter().text(
                    prev_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "‹",
                    FontId::proportional(18.0),
                    COLOR_TEXT_BASE,
                );
            }
            if prev_resp.on_hover_text("Scroll left").clicked() {
                prev_clicked = true;
            }
        });
    });

    (prev_clicked, next_clicked)
}

fn get_time_greeting() -> &'static str {
    "Good evening"
}
