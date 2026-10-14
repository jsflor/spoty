use eframe::egui;
use spoty_core::{PlaylistId, Track};
use spoty_storage::Library;

pub struct PlaylistView;

impl PlaylistView {
    pub fn show(
        ui: &mut egui::Ui,
        library: &Library,
        playlist_id: &PlaylistId,
        current_track_id: Option<&spoty_core::TrackId>,
        on_play_track: &mut impl FnMut(Track),
        on_play_all: &mut impl FnMut(Vec<Track>),
    ) {
        let playlist = match library.playlists.iter().find(|p| &p.id == playlist_id) {
            Some(p) => p,
            None => {
                ui.label("Playlist not found");
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

        // Spotify Hero Banner with vibrant backdrop
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x20, 0x48, 0x38)) // Forest green gradient tone
            .inner_margin(egui::Margin::same(24))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Large playlist artwork box
                    let (rect, _response) = ui.allocate_exact_size(egui::vec2(160.0, 160.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, egui::CornerRadius::same(6), egui::Color32::from_rgb(0x18, 0x18, 0x18));
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🎵",
                        egui::FontId::proportional(48.0),
                        egui::Color32::WHITE,
                    );

                    ui.add_space(20.0);

                    // Hero text details
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("PUBLIC PLAYLIST").size(12.0).strong().color(egui::Color32::GRAY));
                        ui.label(egui::RichText::new(&playlist.title).size(36.0).strong().color(egui::Color32::WHITE));
                        if let Some(desc) = &playlist.description {
                            ui.label(egui::RichText::new(desc).color(egui::Color32::LIGHT_GRAY));
                        }
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "Spoty • {} songs, approx {} min",
                                tracks.len(),
                                total_mins
                            ))
                            .small()
                            .color(egui::Color32::GRAY),
                        );
                    });
                });
            });

        ui.add_space(16.0);

        // Action bar: big green circular play button
        ui.horizontal(|ui| {
            if ui
                .button(
                    egui::RichText::new(" ▶ Play All ")
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::BLACK),
                )
                .clicked()
            {
                on_play_all(tracks.clone());
            }

            ui.add_space(12.0);
            ui.label(egui::RichText::new("♡").size(20.0).color(egui::Color32::GRAY));
            ui.label(egui::RichText::new("•••").size(18.0).color(egui::Color32::GRAY));
        });

        ui.add_space(16.0);

        // Track table
        egui::Grid::new("playlist_tracks_grid")
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("#").strong());
                ui.label(egui::RichText::new("Title").strong());
                ui.label(egui::RichText::new("Artist").strong());
                ui.label(egui::RichText::new("Album").strong());
                ui.label(egui::RichText::new("Duration").strong());
                ui.label(egui::RichText::new("Action").strong());
                ui.end_row();

                for (index, track) in tracks.iter().enumerate() {
                    let is_current = current_track_id == Some(&track.id);
                    let title_color = if is_current {
                        egui::Color32::from_rgb(0x1e, 0xd7, 0x60)
                    } else {
                        egui::Color32::WHITE
                    };

                    ui.label(format!("{}", index + 1));
                    ui.label(egui::RichText::new(&track.title).color(title_color));
                    ui.label(&track.artist);
                    ui.label(track.album.as_deref().unwrap_or("-"));
                    ui.label(track.formatted_duration());

                    if ui.button("▶").clicked() {
                        on_play_track(track.clone());
                    }
                    ui.end_row();
                }
            });
    }
}
