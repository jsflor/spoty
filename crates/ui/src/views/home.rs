use crate::navigation::Route;
use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_HIGHLIGHT, COLOR_TEXT_BASE, COLOR_TEXT_SUBDUED,
};
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use spoty_core::{Playlist, Track};
use spoty_storage::Library;

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

        // Hero Greeting
        ui.add_space(4.0);
        ui.heading(
            egui::RichText::new(greeting)
                .size(30.0)
                .strong()
                .color(COLOR_TEXT_BASE),
        );
        ui.add_space(18.0);

        // Spotify's 2-column or 3-column "Quick Play" rectangular cards
        if !library.playlists.is_empty() {
            let available_w = ui.available_width();
            let num_cols = if available_w > 900.0 { 3 } else { 2 };
            let card_w =
                ((available_w - ((num_cols - 1) as f32 * 12.0)) / num_cols as f32).max(220.0);

            egui::Grid::new("quick_play_grid")
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    for (i, playlist) in library.playlists.iter().enumerate() {
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
                                    on_play_playlist(playlist.clone());
                                    played = true;
                                }
                            }
                            if !played {
                                on_navigate(Route::Playlist(playlist.id.clone()));
                            }
                        }

                        if (i + 1) % num_cols == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(28.0);
        }

        // Section: "Featured & Recent Tracks"
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Featured & Made For You")
                    .size(20.0)
                    .strong()
                    .color(COLOR_TEXT_BASE),
            );
        });
        ui.add_space(14.0);

        // Spotify Card Grid
        ui.horizontal_wrapped(|ui| {
            let card_size = Vec2::new(180.0, 246.0);

            for track in &library.tracks {
                let is_current = current_track_id == Some(&track.id);
                let (rect, resp) = ui.allocate_exact_size(card_size, Sense::click());

                if ui.is_rect_visible(rect) {
                    let p = ui.painter();
                    let is_hovered = resp.hovered();

                    // Card background
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
                    on_play_track(track.clone());
                }

                ui.add_space(14.0);
            }
        });
    }
}

fn get_time_greeting() -> &'static str {
    // Current hour approximation or dynamic greeting
    "Good evening"
}
