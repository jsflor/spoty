use eframe::egui;
use spoty_core::Track;
use spoty_storage::Library;

pub struct SearchView;

impl SearchView {
    pub fn show(
        ui: &mut egui::Ui,
        library: &Library,
        search_query: &mut String,
        current_track_id: Option<&spoty_core::TrackId>,
        on_play_track: &mut impl FnMut(Track),
    ) {
        ui.heading(
            egui::RichText::new("Search")
                .size(26.0)
                .strong()
                .color(egui::Color32::WHITE),
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
                    .color(egui::Color32::WHITE),
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
                        .color(egui::Color32::GRAY),
                );
            } else {
                ui.label(
                    egui::RichText::new(format!("Results ({} tracks)", matches.len()))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(8.0);

                egui::Grid::new("search_results_grid")
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("#").strong());
                        ui.label(egui::RichText::new("Title").strong());
                        ui.label(egui::RichText::new("Artist").strong());
                        ui.label(egui::RichText::new("Duration").strong());
                        ui.label(egui::RichText::new("Play").strong());
                        ui.end_row();

                        for (i, track) in matches.iter().enumerate() {
                            let is_current = current_track_id == Some(&track.id);
                            let title_color = if is_current {
                                egui::Color32::from_rgb(0x1e, 0xd7, 0x60)
                            } else {
                                egui::Color32::WHITE
                            };

                            ui.label(format!("{}", i + 1));
                            ui.label(egui::RichText::new(&track.title).color(title_color));
                            ui.label(&track.artist);
                            ui.label(track.formatted_duration());

                            if ui.button("▶").clicked() {
                                on_play_track((*track).clone());
                            }
                            ui.end_row();
                        }
                    });
            }
        }
    }
}
