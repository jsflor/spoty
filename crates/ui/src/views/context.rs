use crate::navigation::Route;
use spoty_core::{PlaylistId, Track, TrackId};
use spoty_storage::Library;

/// Unified read-only view context provided to views for rendering.
pub struct ViewContext<'a> {
    pub library: &'a Library,
    pub current_track_id: Option<&'a TrackId>,
    pub is_favorite: &'a dyn Fn(&TrackId) -> bool,
}

impl<'a> ViewContext<'a> {
    pub fn new(
        library: &'a Library,
        current_track_id: Option<&'a TrackId>,
        is_favorite: &'a dyn Fn(&TrackId) -> bool,
    ) -> Self {
        Self {
            library,
            current_track_id,
            is_favorite,
        }
    }
}

/// Strongly-typed actions emitted by immediate-mode views to express user intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewAction {
    Navigate(Route),
    PlayTrack(Track),
    PlayAll(Vec<Track>),
    ToggleFavorite(TrackId),
    AddToQueue(Track),
    AddToPlaylist(PlaylistId, TrackId),
    CreatePlaylistWithTrack(TrackId),
    EditPlaylist(PlaylistId),
    DeletePlaylist(PlaylistId),
    RemoveTrackFromPlaylist(PlaylistId, TrackId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_context_construction() {
        let library = Library::default();
        let track_id = TrackId::new("trk_1");
        let is_fav = |id: &TrackId| id.as_str() == "trk_1";

        let ctx = ViewContext::new(&library, Some(&track_id), &is_fav);
        assert_eq!(ctx.current_track_id, Some(&track_id));
        assert!((ctx.is_favorite)(&track_id));
        assert!(!(ctx.is_favorite)(&TrackId::new("trk_other")));
    }

    #[test]
    fn test_view_action_equality_and_cloning() {
        let action1 = ViewAction::ToggleFavorite(TrackId::new("t1"));
        let action2 = action1.clone();
        assert_eq!(action1, action2);

        let nav = ViewAction::Navigate(Route::Home);
        assert_ne!(action1, nav);
    }
}
