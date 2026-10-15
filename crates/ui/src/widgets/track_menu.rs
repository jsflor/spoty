use crate::views::ViewAction;
use eframe::egui::{self, Color32};
use spoty_core::{Playlist, PlaylistId, Track};

pub fn render_track_context_menu(
    ui: &mut egui::Ui,
    track: &Track,
    custom_playlists: &[Playlist],
    current_playlist_id: Option<&PlaylistId>,
    is_fav: bool,
) -> Option<ViewAction> {
    ui.set_min_width(200.0);
    let mut action = None;

    // 1. Add to Queue
    if ui.button("➕  Add to Queue").clicked() {
        action = Some(ViewAction::AddToQueue(track.clone()));
        ui.close_menu();
    }

    // 2. Favorite toggle
    if is_fav {
        if ui.button("💚  Remove from Liked Songs").clicked() {
            action = Some(ViewAction::ToggleFavorite(track.id.clone()));
            ui.close_menu();
        }
    } else if ui.button("🤍  Save to Liked Songs").clicked() {
        action = Some(ViewAction::ToggleFavorite(track.id.clone()));
        ui.close_menu();
    }

    ui.separator();

    // 3. Add to Playlist
    ui.menu_button("📁  Add to Playlist", |ui| {
        ui.set_min_width(180.0);
        if ui.button("+  New playlist").clicked() {
            action = Some(ViewAction::CreatePlaylistWithTrack(track.id.clone()));
            ui.close_menu();
        }

        if !custom_playlists.is_empty() {
            ui.separator();
            for pl in custom_playlists {
                let already_contains = pl.track_ids.contains(&track.id);
                let text = if already_contains {
                    format!("✓  {}", pl.title)
                } else {
                    pl.title.clone()
                };
                if ui.button(text).clicked() {
                    action = Some(ViewAction::AddToPlaylist(pl.id.clone(), track.id.clone()));
                    ui.close_menu();
                }
            }
        }
    });

    // 4. Remove from this playlist (if viewing a custom playlist)
    if let Some(cur_pl_id) = current_playlist_id
        && cur_pl_id.as_str().starts_with("custom_pl_")
    {
        ui.separator();
        if ui
            .button(
                egui::RichText::new("🗑  Remove from this playlist")
                    .color(Color32::from_rgb(0xff, 0x60, 0x60)),
            )
            .clicked()
        {
            action = Some(ViewAction::RemoveTrackFromPlaylist(
                cur_pl_id.clone(),
                track.id.clone(),
            ));
            ui.close_menu();
        }
    }

    action
}
