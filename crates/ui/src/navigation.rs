use spoty_core::{AlbumId, PlaylistId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Home,
    Search,
    AllSongs,
    LikedSongs,
    Playlist(PlaylistId),
    #[allow(dead_code)]
    Album(AlbumId),
    Settings,
}

impl Default for Route {
    fn default() -> Self {
        Self::Home
    }
}

#[derive(Debug, Clone)]
pub struct NavigationHistory {
    stack: Vec<Route>,
    cursor: usize,
}

impl Default for NavigationHistory {
    fn default() -> Self {
        Self {
            stack: vec![Route::Home],
            cursor: 0,
        }
    }
}

impl NavigationHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> &Route {
        &self.stack[self.cursor]
    }

    pub fn can_go_back(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.cursor + 1 < self.stack.len()
    }

    pub fn back(&mut self) -> &Route {
        if self.can_go_back() {
            self.cursor -= 1;
        }
        self.current()
    }

    pub fn forward(&mut self) -> &Route {
        if self.can_go_forward() {
            self.cursor += 1;
        }
        self.current()
    }

    pub fn push(&mut self, route: Route) {
        if self.current() == &route {
            return;
        }
        // Truncate forward history if navigating from a past point
        self.stack.truncate(self.cursor + 1);
        self.stack.push(route);
        self.cursor = self.stack.len() - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history_stack() {
        let mut nav = NavigationHistory::new();
        assert_eq!(nav.current(), &Route::Home);
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());

        nav.push(Route::Search);
        assert_eq!(nav.current(), &Route::Search);
        assert!(nav.can_go_back());
        assert!(!nav.can_go_forward());

        nav.back();
        assert_eq!(nav.current(), &Route::Home);
        assert!(!nav.can_go_back());
        assert!(nav.can_go_forward());

        nav.forward();
        assert_eq!(nav.current(), &Route::Search);

        nav.push(Route::AllSongs);
        assert_eq!(nav.current(), &Route::AllSongs);

        nav.push(Route::LikedSongs);
        assert_eq!(nav.current(), &Route::LikedSongs);
        assert!(nav.can_go_back());
    }
}
