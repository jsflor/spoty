use crate::model::{RepeatMode, Track};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Queue {
    tracks: Vec<Track>,
    current_index: Option<usize>,
    shuffle: bool,
    shuffle_order: Vec<usize>,
}

impl Queue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tracks(tracks: Vec<Track>, start_index: Option<usize>) -> Self {
        let mut queue = Self {
            tracks,
            current_index: None,
            shuffle: false,
            shuffle_order: Vec::new(),
        };
        queue.generate_shuffle_order();
        if let Some(idx) = start_index {
            queue.jump_to(idx);
        }
        queue
    }

    pub fn set_tracks(&mut self, tracks: Vec<Track>, start_index: Option<usize>) {
        self.tracks = tracks;
        self.generate_shuffle_order();
        if let Some(idx) = start_index {
            self.jump_to(idx);
        } else if !self.tracks.is_empty() {
            self.jump_to(0);
        } else {
            self.current_index = None;
        }
    }

    pub fn current_track(&self) -> Option<&Track> {
        let idx = self.current_index?;
        self.tracks.get(idx)
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn is_shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn set_shuffle(&mut self, shuffle: bool) {
        self.shuffle = shuffle;
        if shuffle {
            self.generate_shuffle_order();
        }
    }

    pub fn toggle_shuffle(&mut self) -> bool {
        self.set_shuffle(!self.shuffle);
        self.shuffle
    }

    pub fn jump_to(&mut self, index: usize) -> Option<&Track> {
        if index < self.tracks.len() {
            self.current_index = Some(index);
            self.tracks.get(index)
        } else {
            None
        }
    }

    pub fn next(&mut self, repeat: RepeatMode) -> Option<&Track> {
        if self.tracks.is_empty() {
            return None;
        }

        if repeat == RepeatMode::One {
            return self.current_track();
        }

        let total = self.tracks.len();

        if self.shuffle && !self.shuffle_order.is_empty() {
            // Traverse shuffle order
            let current_shuffle_pos = self
                .current_index
                .and_then(|curr| self.shuffle_order.iter().position(|&idx| idx == curr))
                .unwrap_or(0);

            let next_shuffle_pos = current_shuffle_pos + 1;
            if next_shuffle_pos < self.shuffle_order.len() {
                let next_idx = self.shuffle_order[next_shuffle_pos];
                self.current_index = Some(next_idx);
                return self.tracks.get(next_idx);
            } else if repeat == RepeatMode::All {
                // Loop back to beginning of shuffle order
                let next_idx = self.shuffle_order[0];
                self.current_index = Some(next_idx);
                return self.tracks.get(next_idx);
            } else {
                return None;
            }
        }

        // Sequential traversal
        let next_idx = match self.current_index {
            Some(curr) => {
                if curr + 1 < total {
                    curr + 1
                } else if repeat == RepeatMode::All {
                    0
                } else {
                    return None;
                }
            }
            None => 0,
        };

        self.current_index = Some(next_idx);
        self.tracks.get(next_idx)
    }

    pub fn prev(&mut self) -> Option<&Track> {
        if self.tracks.is_empty() {
            return None;
        }

        let next_idx = match self.current_index {
            Some(curr) if curr > 0 => curr - 1,
            _ => 0,
        };

        self.current_index = Some(next_idx);
        self.tracks.get(next_idx)
    }

    fn generate_shuffle_order(&mut self) {
        let n = self.tracks.len();
        let mut order: Vec<usize> = (0..n).collect();
        // Simple deterministic Fisher-Yates shuffle
        let mut seed = 0x517cc1b727220a95u64;
        for i in (1..n).rev() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (seed % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        self.shuffle_order = order;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::TrackId;
    use crate::model::AudioFormat;

    fn make_test_track(id: &str, title: &str) -> Track {
        Track {
            id: TrackId::new(id),
            title: title.into(),
            artist: "Artist".into(),
            album: None,
            duration_secs: 180,
            cover_uri: None,
            source_uri: format!("demo://{}", id),
            format: AudioFormat::Wav,
        }
    }

    #[test]
    fn test_queue_sequential_navigation() {
        let tracks = vec![
            make_test_track("1", "Song 1"),
            make_test_track("2", "Song 2"),
            make_test_track("3", "Song 3"),
        ];

        let mut queue = Queue::with_tracks(tracks, Some(0));
        assert_eq!(queue.current_track().unwrap().title, "Song 1");

        // Next
        let t2 = queue.next(RepeatMode::Off).unwrap();
        assert_eq!(t2.title, "Song 2");

        let t3 = queue.next(RepeatMode::Off).unwrap();
        assert_eq!(t3.title, "Song 3");

        // End reached with RepeatMode::Off
        assert!(queue.next(RepeatMode::Off).is_none());

        // Previous
        let p2 = queue.prev().unwrap();
        assert_eq!(p2.title, "Song 2");
    }

    #[test]
    fn test_queue_repeat_all() {
        let tracks = vec![
            make_test_track("1", "Song 1"),
            make_test_track("2", "Song 2"),
        ];

        let mut queue = Queue::with_tracks(tracks, Some(1));
        assert_eq!(queue.current_track().unwrap().title, "Song 2");

        // Next wraps around to 0
        let wrapped = queue.next(RepeatMode::All).unwrap();
        assert_eq!(wrapped.title, "Song 1");
    }

    #[test]
    fn test_queue_repeat_one() {
        let tracks = vec![
            make_test_track("1", "Song 1"),
            make_test_track("2", "Song 2"),
        ];

        let mut queue = Queue::with_tracks(tracks, Some(0));
        let repeated = queue.next(RepeatMode::One).unwrap();
        assert_eq!(repeated.title, "Song 1");
    }
}
