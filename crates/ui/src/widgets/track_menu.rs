use eframe::egui::{self, Color32};
use spoty_core::{Playlist, PlaylistId, Track, TrackId};

pub struct TrackMenuActions<'a> {
    pub is_fav: bool,
    pub custom_playlists: &'a [Playlist],
    pub current_playlist_id: Option<&'a PlaylistId>,
    pub on_toggle_fav: &'a mut dyn FnMut(TrackId),
    pub on_add_to_queue: &'a mut dyn FnMut(Track),
    pub on_add_to_playlist: &'a mut dyn FnMut(PlaylistId, TrackId),
    pub on_create_playlist_with_track: &'a mut dyn FnMut(TrackId),
    pub on_remove_from_playlist: Option<&'a mut dyn FnMut(PlaylistId, TrackId)>,
}

pub fn render_track_context_menu(
    ui: &mut egui::Ui,
    track: &Track,
    actions: &mut TrackMenuActions<'_>,
) {
    ui.set_min_width(200.0);

    // 1. Add to Queue
    if ui.button("➕  Add to Queue").clicked() {
        (actions.on_add_to_queue)(track.clone());
        ui.close_menu();
    }

    // 2. Favorite toggle
    if actions.is_fav {
        if ui.button("💚  Remove from Liked Songs").clicked() {
            (actions.on_toggle_fav)(track.id.clone());
            ui.close_menu();
        }
    } else if ui.button("🤍  Save to Liked Songs").clicked() {
        (actions.on_toggle_fav)(track.id.clone());
        ui.close_menu();
    }

    ui.separator();

    // 3. Add to Playlist
    ui.menu_button("📁  Add to Playlist", |ui| {
        ui.set_min_width(180.0);
        if ui.button("+  New playlist").clicked() {
            (actions.on_create_playlist_with_track)(track.id.clone());
            ui.close_menu();
        }

        if !actions.custom_playlists.is_empty() {
            ui.separator();
            for pl in actions.custom_playlists {
                let already_contains = pl.track_ids.contains(&track.id);
                let text = if already_contains {
                    format!("✓  {}", pl.title)
                } else {
                    pl.title.clone()
                };
                if ui.button(text).clicked() {
                    (actions.on_add_to_playlist)(pl.id.clone(), track.id.clone());
                    ui.close_menu();
                }
            }
        }
    });

    // 4. Remove from this playlist (if viewing a custom playlist)
    if let (Some(cur_pl_id), Some(on_remove)) = (
        actions.current_playlist_id,
        actions.on_remove_from_playlist.as_mut(),
    ) {
        if cur_pl_id.as_str().starts_with("custom_pl_") {
            ui.separator();
            if ui
                .button(
                    egui::RichText::new("🗑  Remove from this playlist")
                        .color(Color32::from_rgb(0xff, 0x60, 0x60)),
                )
                .clicked()
            {
                on_remove((*cur_pl_id).clone(), track.id.clone());
                ui.close_menu();
            }
        }
    }
}
