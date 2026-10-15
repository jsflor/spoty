use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_BG_CARD, COLOR_BG_ELEVATED, COLOR_BG_HIGHLIGHT, COLOR_DIVIDER,
    COLOR_TEXT_BASE, COLOR_TEXT_SUBDUED,
};
use crate::views::{ViewAction, ViewContext};
use crate::widgets::{draw_sort_arrow, play_circle_button, render_track_context_menu};
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use spoty_core::Track;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SongsMode {
    All,
    Liked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    #[default]
    Index,
    Title,
    Artist,
    Album,
    Duration,
}

#[derive(Debug, Clone, Default)]
pub struct SongsViewState {
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub filter_query: String,
}

pub struct SongsView;

impl SongsView {
    pub fn show(ui: &mut egui::Ui, ctx: &ViewContext<'_>, mode: SongsMode) -> Option<ViewAction> {
        let mut action = None;

        let mut tracks: Vec<Track> = match mode {
            SongsMode::All => ctx.library.tracks.clone(),
            SongsMode::Liked => ctx
                .library
                .tracks
                .iter()
                .filter(|t| (ctx.is_favorite)(&t.id))
                .cloned()
                .collect(),
        };

        let view_id = ui.make_persistent_id(match mode {
            SongsMode::All => "all_songs_view_state",
            SongsMode::Liked => "liked_songs_view_state",
        });
        let mut state = ui.data_mut(|d| d.get_temp::<SongsViewState>(view_id).unwrap_or_default());

        // Instant filter
        if !state.filter_query.trim().is_empty() {
            let q = state.filter_query.trim().to_lowercase();
            tracks.retain(|t| {
                t.title.to_lowercase().contains(&q)
                    || t.artist.to_lowercase().contains(&q)
                    || t.album.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || t.source_uri.to_lowercase().contains(&q)
            });
        }

        // Column sorting
        match state.sort_column {
            SortColumn::Index => {
                if !state.sort_ascending {
                    tracks.reverse();
                }
            }
            SortColumn::Title => {
                tracks.sort_by(|a, b| {
                    let ord = a.title.to_lowercase().cmp(&b.title.to_lowercase());
                    if state.sort_ascending {
                        ord
                    } else {
                        ord.reverse()
                    }
                });
            }
            SortColumn::Artist => {
                tracks.sort_by(|a, b| {
                    let ord = a.artist.to_lowercase().cmp(&b.artist.to_lowercase());
                    if state.sort_ascending {
                        ord
                    } else {
                        ord.reverse()
                    }
                });
            }
            SortColumn::Album => {
                tracks.sort_by(|a, b| {
                    let album_a = a.album.as_deref().unwrap_or("").to_lowercase();
                    let album_b = b.album.as_deref().unwrap_or("").to_lowercase();
                    let ord = album_a.cmp(&album_b);
                    if state.sort_ascending {
                        ord
                    } else {
                        ord.reverse()
                    }
                });
            }
            SortColumn::Duration => {
                tracks.sort_by(|a, b| {
                    let ord = a.duration_secs.cmp(&b.duration_secs);
                    if state.sort_ascending {
                        ord
                    } else {
                        ord.reverse()
                    }
                });
            }
        }

        let total_duration_secs: u32 = tracks.iter().map(|t| t.duration_secs).sum();
        let total_mins = total_duration_secs / 60;

        // 1. Full-Bleed Hero Banner
        let (hero_title, hero_bg, hero_icon) = match mode {
            SongsMode::All => (
                "All Songs",
                Color32::from_rgb(0x1a, 0x2e, 0x3d), // Deep navy
                "🎶",
            ),
            SongsMode::Liked => (
                "Liked Songs",
                Color32::from_rgb(0x2d, 0x1a, 0x47), // Deep purple
                "💚",
            ),
        };

        egui::Frame::new()
            .fill(hero_bg)
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(24))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Artwork Box
                    let art_size = Vec2::splat(140.0);
                    let (art_rect, _) = ui.allocate_exact_size(art_size, Sense::hover());
                    if ui.is_rect_visible(art_rect) {
                        let p = ui.painter();
                        p.rect_filled(art_rect, CornerRadius::same(6), COLOR_BG_CARD);
                        p.circle_stroke(
                            art_rect.center(),
                            36.0,
                            Stroke::new(1.0_f32, Color32::from_black_alpha(40)),
                        );
                        p.text(
                            art_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            hero_icon,
                            FontId::proportional(50.0),
                            COLOR_TEXT_BASE,
                        );
                    }

                    ui.add_space(20.0);

                    // Hero Text
                    ui.vertical(|ui| {
                        let tag = match mode {
                            SongsMode::All => "LIBRARY",
                            SongsMode::Liked => "PLAYLIST",
                        };
                        ui.label(
                            egui::RichText::new(tag)
                                .size(11.0)
                                .strong()
                                .color(COLOR_TEXT_BASE),
                        );
                        ui.add_space(2.0);

                        ui.label(
                            egui::RichText::new(hero_title)
                                .size(38.0)
                                .strong()
                                .color(COLOR_TEXT_BASE),
                        );
                        ui.add_space(10.0);

                        let summary = if !state.filter_query.trim().is_empty() {
                            format!(
                                "Showing {} matching songs (approx {} min)",
                                tracks.len(),
                                total_mins
                            )
                        } else {
                            format!("{} songs, approx {} min", tracks.len(), total_mins)
                        };
                        ui.label(
                            egui::RichText::new(summary)
                                .size(12.0)
                                .color(COLOR_TEXT_SUBDUED),
                        );
                    });
                });
            });

        ui.add_space(18.0);

        // 2. Action Bar: Play Button + In-View Search Filter
        ui.horizontal(|ui| {
            if play_circle_button(ui, 48.0, false, Some(COLOR_ACCENT_GREEN))
                .on_hover_text("Play All")
                .clicked()
            {
                action = Some(ViewAction::PlayAll(tracks.clone()));
            }

            ui.add_space(16.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::Frame::new()
                    .fill(COLOR_BG_ELEVATED)
                    .corner_radius(CornerRadius::same(20))
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("🔍")
                                    .size(12.0)
                                    .color(COLOR_TEXT_SUBDUED),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut state.filter_query)
                                    .hint_text("Filter in songs...")
                                    .desired_width(180.0)
                                    .frame(false),
                            );
                        });
                    });
            });
        });

        ui.add_space(18.0);

        // 3. Interactive Track List Table Header
        let table_width = ui.available_width();
        let (hdr_rect, _) = ui.allocate_exact_size(Vec2::new(table_width, 28.0), Sense::hover());
        if ui.is_rect_visible(hdr_rect) {
            let cols = [
                (
                    Rect::from_min_size(
                        Pos2::new(hdr_rect.left() + 8.0, hdr_rect.top()),
                        Vec2::new(28.0, 28.0),
                    ),
                    "#",
                    SortColumn::Index,
                    egui::Align2::CENTER_CENTER,
                ),
                (
                    Rect::from_min_size(
                        Pos2::new(hdr_rect.left() + 48.0, hdr_rect.top()),
                        Vec2::new((table_width * 0.42).max(140.0), 28.0),
                    ),
                    "Title",
                    SortColumn::Title,
                    egui::Align2::LEFT_CENTER,
                ),
                (
                    Rect::from_min_size(
                        Pos2::new(hdr_rect.left() + (table_width * 0.42), hdr_rect.top()),
                        Vec2::new(140.0, 28.0),
                    ),
                    "Artist",
                    SortColumn::Artist,
                    egui::Align2::LEFT_CENTER,
                ),
                (
                    Rect::from_min_size(
                        Pos2::new(hdr_rect.left() + (table_width * 0.68), hdr_rect.top()),
                        Vec2::new(140.0, 28.0),
                    ),
                    "Album",
                    SortColumn::Album,
                    egui::Align2::LEFT_CENTER,
                ),
                (
                    Rect::from_min_size(
                        Pos2::new(hdr_rect.right() - 56.0, hdr_rect.top()),
                        Vec2::new(40.0, 28.0),
                    ),
                    "🕒",
                    SortColumn::Duration,
                    egui::Align2::CENTER_CENTER,
                ),
            ];

            for (rect, label, col, align) in cols {
                let resp = ui.allocate_rect(rect, Sense::click());
                let is_sorted = state.sort_column == col;

                let color = if resp.hovered() || is_sorted {
                    COLOR_TEXT_BASE
                } else {
                    COLOR_TEXT_SUBDUED
                };

                let pos = match align {
                    egui::Align2::CENTER_CENTER => rect.center(),
                    egui::Align2::LEFT_CENTER => Pos2::new(rect.left(), rect.center().y),
                    egui::Align2::RIGHT_CENTER => Pos2::new(rect.right(), rect.center().y),
                    _ => rect.center(),
                };

                let galley = ui.painter().layout_no_wrap(
                    label.to_string(),
                    FontId::proportional(12.0),
                    color,
                );
                let text_pos = match align {
                    egui::Align2::CENTER_CENTER => Pos2::new(
                        pos.x - (galley.size().x / 2.0),
                        pos.y - (galley.size().y / 2.0),
                    ),
                    egui::Align2::LEFT_CENTER => Pos2::new(pos.x, pos.y - (galley.size().y / 2.0)),
                    egui::Align2::RIGHT_CENTER => {
                        Pos2::new(pos.x - galley.size().x, pos.y - (galley.size().y / 2.0))
                    }
                    _ => pos,
                };
                ui.painter().galley(text_pos, galley.clone(), color);

                if is_sorted {
                    let arrow_x = text_pos.x + galley.size().x + 7.0;
                    let arrow_center = Pos2::new(arrow_x, rect.center().y);
                    draw_sort_arrow(ui.painter(), arrow_center, state.sort_ascending, 7.0, color);
                }

                if resp.clicked() {
                    if state.sort_column == col {
                        state.sort_ascending = !state.sort_ascending;
                    } else {
                        state.sort_column = col;
                        state.sort_ascending = true;
                    }
                }
            }

            // Divider line
            ui.painter().line_segment(
                [
                    Pos2::new(hdr_rect.left(), hdr_rect.bottom()),
                    Pos2::new(hdr_rect.right(), hdr_rect.bottom()),
                ],
                Stroke::new(1.0_f32, COLOR_DIVIDER),
            );
        }

        ui.add_space(6.0);

        if tracks.is_empty() {
            ui.add_space(20.0);
            let empty_text = match mode {
                SongsMode::All => "No tracks found in library.",
                SongsMode::Liked => {
                    "No liked songs yet. Click the heart icon on any track to save it here!"
                }
            };
            ui.label(
                egui::RichText::new(empty_text)
                    .size(13.0)
                    .color(COLOR_TEXT_SUBDUED),
            );
        } else {
            // 4. Interactive Track List Rows
            for (index, track) in tracks.iter().enumerate() {
                let is_current = ctx.current_track_id == Some(&track.id);
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
                    let idx_pos = Pos2::new(row_rect.left() + 22.0, row_rect.center().y);
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
                        Pos2::new(row_rect.left() + 42.0, row_rect.min.y + 7.0),
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
                        Pos2::new(row_rect.left() + 90.0, row_rect.center().y),
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
                    action = Some(ViewAction::PlayTrack(track.clone()));
                }

                let fav = (ctx.is_favorite)(&track.id);

                row_resp.context_menu(|ui| {
                    if let Some(act) =
                        render_track_context_menu(ui, track, &ctx.library.playlists, None, fav)
                    {
                        action = Some(act);
                    }
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
                    action = Some(ViewAction::ToggleFavorite(track.id.clone()));
                }

                ui.add_space(2.0);
            }
        }

        ui.data_mut(|d| d.insert_temp(view_id, state));
        action
    }
}
