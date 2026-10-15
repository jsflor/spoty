use crate::engine::{PlayerCommand, PlayerEvent};
use crate::reader::AudioStreamReader;
use crate::worker::AudioWorker;
use spoty_core::{PlaybackState, PlaybackStatus, Queue, RepeatMode, Track, TrackId};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Interface for resolving an `AudioStreamReader` for a track.
pub trait AudioStreamResolver: Send + Sync + 'static {
    fn resolve_stream(&self, track: &Track) -> anyhow::Result<Box<dyn AudioStreamReader>>;
}

/// An authoritative snapshot of playback and queue state.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionState {
    pub playback: PlaybackState,
    pub current_track: Option<Track>,
    pub upcoming_tracks: Vec<Track>,
    pub is_shuffle: bool,
    pub repeat_mode: RepeatMode,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            playback: PlaybackState::default(),
            current_track: None,
            upcoming_tracks: Vec::new(),
            is_shuffle: false,
            repeat_mode: RepeatMode::Off,
        }
    }
}

/// Non-blocking commands dispatched to `PlaybackSession`.
pub enum SessionCommand {
    PlayTrack(TrackId),
    PlayAll {
        tracks: Vec<Track>,
        start_index: usize,
    },
    TogglePlayPause,
    Play,
    Pause,
    Next,
    Prev,
    Seek(Duration),
    SetVolume(f32),
    ToggleMute,
    ToggleShuffle,
    CycleRepeatMode,
    SetQueue(Vec<Track>, Option<usize>),
    EnqueueNext(Track),
    EnqueueLast(Track),
    RemoveFromQueue(usize),
    ClearUpcoming,
    SetResolver(Arc<dyn AudioStreamResolver>),
}

impl std::fmt::Debug for SessionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayTrack(id) => f.debug_tuple("PlayTrack").field(id).finish(),
            Self::PlayAll {
                tracks,
                start_index,
            } => f
                .debug_struct("PlayAll")
                .field("tracks", tracks)
                .field("start_index", start_index)
                .finish(),
            Self::TogglePlayPause => write!(f, "TogglePlayPause"),
            Self::Play => write!(f, "Play"),
            Self::Pause => write!(f, "Pause"),
            Self::Next => write!(f, "Next"),
            Self::Prev => write!(f, "Prev"),
            Self::Seek(d) => f.debug_tuple("Seek").field(d).finish(),
            Self::SetVolume(v) => f.debug_tuple("SetVolume").field(v).finish(),
            Self::ToggleMute => write!(f, "ToggleMute"),
            Self::ToggleShuffle => write!(f, "ToggleShuffle"),
            Self::CycleRepeatMode => write!(f, "CycleRepeatMode"),
            Self::SetQueue(tracks, start_index) => f
                .debug_struct("SetQueue")
                .field("tracks", tracks)
                .field("start_index", start_index)
                .finish(),
            Self::EnqueueNext(t) => f.debug_tuple("EnqueueNext").field(t).finish(),
            Self::EnqueueLast(t) => f.debug_tuple("EnqueueLast").field(t).finish(),
            Self::RemoveFromQueue(idx) => f.debug_tuple("RemoveFromQueue").field(idx).finish(),
            Self::ClearUpcoming => write!(f, "ClearUpcoming"),
            Self::SetResolver(_) => write!(f, "SetResolver(<resolver>)"),
        }
    }
}

/// Events emitted by `PlaybackSession` to subscribers (e.g. the UI).
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum SessionEvent {
    StateChanged(SessionState),
    VisualizerLevels([f32; 16]),
    Error(String),
}

/// A deep playback coordinator managing queue progression, stream ingestion, and audio output.
#[derive(Clone)]
pub struct PlaybackSession {
    cmd_tx: Sender<SessionCommand>,
    cached_state: Arc<RwLock<SessionState>>,
}

impl PlaybackSession {
    pub fn new(resolver: Arc<dyn AudioStreamResolver>) -> (Self, Receiver<SessionEvent>) {
        Self::with_queue(resolver, Queue::new())
    }

    pub fn with_queue(
        resolver: Arc<dyn AudioStreamResolver>,
        queue: Queue,
    ) -> (Self, Receiver<SessionEvent>) {
        let (session_cmd_tx, session_cmd_rx) = channel();
        let (session_event_tx, session_event_rx) = channel();
        let (worker_cmd_tx, worker_cmd_rx) = channel();
        let (worker_event_tx, worker_event_rx) = channel();

        AudioWorker::spawn(worker_cmd_rx, worker_event_tx);

        let initial_state = SessionState {
            playback: PlaybackState::default(),
            current_track: queue.current_track().cloned(),
            upcoming_tracks: queue
                .upcoming_tracks()
                .into_iter()
                .map(|(_, t)| t.clone())
                .collect(),
            is_shuffle: queue.is_shuffle(),
            repeat_mode: RepeatMode::Off,
        };

        let cached_state = Arc::new(RwLock::new(initial_state));
        let cached_state_clone = Arc::clone(&cached_state);

        std::thread::Builder::new()
            .name("spoty-session-coordinator".into())
            .spawn(move || {
                let mut coordinator = SessionCoordinator {
                    resolver,
                    queue,
                    repeat_mode: RepeatMode::Off,
                    cached_state: cached_state_clone,
                    cmd_rx: session_cmd_rx,
                    event_tx: session_event_tx,
                    worker_tx: worker_cmd_tx,
                    worker_rx: worker_event_rx,
                };
                coordinator.run();
            })
            .expect("Failed to spawn session coordinator thread");

        (
            Self {
                cmd_tx: session_cmd_tx,
                cached_state,
            },
            session_event_rx,
        )
    }

    pub fn state(&self) -> SessionState {
        self.cached_state.read().unwrap().clone()
    }

    pub fn play_track(&self, id: TrackId) {
        let _ = self.cmd_tx.send(SessionCommand::PlayTrack(id));
    }

    pub fn play_all(&self, tracks: Vec<Track>, start_index: usize) {
        let _ = self.cmd_tx.send(SessionCommand::PlayAll {
            tracks,
            start_index,
        });
    }

    pub fn toggle_play_pause(&self) {
        let _ = self.cmd_tx.send(SessionCommand::TogglePlayPause);
    }

    pub fn play(&self) {
        let _ = self.cmd_tx.send(SessionCommand::Play);
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(SessionCommand::Pause);
    }

    pub fn next(&self) {
        let _ = self.cmd_tx.send(SessionCommand::Next);
    }

    pub fn prev(&self) {
        let _ = self.cmd_tx.send(SessionCommand::Prev);
    }

    pub fn seek(&self, position: Duration) {
        if let Ok(mut lock) = self.cached_state.write() {
            lock.playback.position_secs = position.as_secs_f64();
        }
        let _ = self.cmd_tx.send(SessionCommand::Seek(position));
    }

    pub fn set_volume(&self, volume: f32) {
        if let Ok(mut lock) = self.cached_state.write() {
            lock.playback.volume = volume.clamp(0.0, 1.0);
            if volume > 0.0 {
                lock.playback.is_muted = false;
            }
        }
        let _ = self.cmd_tx.send(SessionCommand::SetVolume(volume));
    }

    pub fn toggle_mute(&self) {
        if let Ok(mut lock) = self.cached_state.write() {
            lock.playback.is_muted = !lock.playback.is_muted;
        }
        let _ = self.cmd_tx.send(SessionCommand::ToggleMute);
    }

    pub fn toggle_shuffle(&self) {
        let _ = self.cmd_tx.send(SessionCommand::ToggleShuffle);
    }

    pub fn cycle_repeat_mode(&self) {
        let _ = self.cmd_tx.send(SessionCommand::CycleRepeatMode);
    }

    pub fn set_queue(&self, tracks: Vec<Track>, start_index: Option<usize>) {
        let _ = self
            .cmd_tx
            .send(SessionCommand::SetQueue(tracks, start_index));
    }

    pub fn enqueue_next(&self, track: Track) {
        let _ = self.cmd_tx.send(SessionCommand::EnqueueNext(track));
    }

    pub fn enqueue_last(&self, track: Track) {
        let _ = self.cmd_tx.send(SessionCommand::EnqueueLast(track));
    }

    pub fn remove_from_queue(&self, index: usize) {
        let _ = self.cmd_tx.send(SessionCommand::RemoveFromQueue(index));
    }

    pub fn clear_upcoming(&self) {
        let _ = self.cmd_tx.send(SessionCommand::ClearUpcoming);
    }

    pub fn set_resolver(&self, resolver: Arc<dyn AudioStreamResolver>) {
        let _ = self.cmd_tx.send(SessionCommand::SetResolver(resolver));
    }
}

struct SessionCoordinator {
    resolver: Arc<dyn AudioStreamResolver>,
    queue: Queue,
    repeat_mode: RepeatMode,
    cached_state: Arc<RwLock<SessionState>>,
    cmd_rx: Receiver<SessionCommand>,
    event_tx: Sender<SessionEvent>,
    worker_tx: Sender<PlayerCommand>,
    worker_rx: Receiver<PlayerEvent>,
}

impl SessionCoordinator {
    fn run(&mut self) {
        loop {
            match self.cmd_rx.recv_timeout(Duration::from_millis(20)) {
                Ok(cmd) => self.handle_session_command(cmd),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }

            while let Ok(event) = self.worker_rx.try_recv() {
                self.handle_worker_event(event);
            }
        }
    }

    fn handle_session_command(&mut self, cmd: SessionCommand) {
        match cmd {
            SessionCommand::PlayTrack(track_id) => {
                if let Some(pos) = self.queue.tracks().iter().position(|t| t.id == track_id) {
                    self.queue.jump_to(pos);
                }
                self.load_and_play_current(Duration::ZERO);
            }
            SessionCommand::PlayAll {
                tracks,
                start_index,
            } => {
                self.queue.set_tracks(tracks, Some(start_index));
                self.load_and_play_current(Duration::ZERO);
            }
            SessionCommand::TogglePlayPause => {
                let _ = self.worker_tx.send(PlayerCommand::TogglePlayPause);
            }
            SessionCommand::Play => {
                let _ = self.worker_tx.send(PlayerCommand::Play);
            }
            SessionCommand::Pause => {
                let _ = self.worker_tx.send(PlayerCommand::Pause);
            }
            SessionCommand::Next => {
                self.advance_next();
            }
            SessionCommand::Prev => {
                let pos_secs = self.cached_state.read().unwrap().playback.position_secs;
                if pos_secs > 3.0 {
                    self.seek(Duration::ZERO);
                } else if self.queue.prev().is_some() {
                    self.load_and_play_current(Duration::ZERO);
                }
            }
            SessionCommand::Seek(pos) => {
                self.seek(pos);
            }
            SessionCommand::SetVolume(vol) => {
                let _ = self.worker_tx.send(PlayerCommand::SetVolume(vol));
            }
            SessionCommand::ToggleMute => {
                let _ = self.worker_tx.send(PlayerCommand::ToggleMute);
            }
            SessionCommand::ToggleShuffle => {
                self.queue.toggle_shuffle();
                let _ = self.worker_tx.send(PlayerCommand::ToggleShuffle);
                self.sync_and_emit();
            }
            SessionCommand::CycleRepeatMode => {
                self.repeat_mode = match self.repeat_mode {
                    RepeatMode::Off => RepeatMode::All,
                    RepeatMode::All => RepeatMode::One,
                    RepeatMode::One => RepeatMode::Off,
                };
                let _ = self.worker_tx.send(PlayerCommand::CycleRepeatMode);
                self.sync_and_emit();
            }
            SessionCommand::SetQueue(tracks, start_index) => {
                self.queue.set_tracks(tracks, start_index);
                self.sync_and_emit();
            }
            SessionCommand::EnqueueNext(track) => {
                self.queue.insert_next(track);
                self.sync_and_emit();
            }
            SessionCommand::EnqueueLast(track) => {
                self.queue.push_back(track);
                self.sync_and_emit();
            }
            SessionCommand::RemoveFromQueue(idx) => {
                let upcoming = self.queue.upcoming_tracks();
                if idx < upcoming.len() {
                    let (actual_idx, _) = upcoming[idx];
                    self.queue.remove_at(actual_idx);
                } else {
                    self.queue.remove_at(idx);
                }
                self.sync_and_emit();
            }
            SessionCommand::ClearUpcoming => {
                self.queue.clear_upcoming();
                self.sync_and_emit();
            }
            SessionCommand::SetResolver(resolver) => {
                self.resolver = resolver;
            }
        }
    }

    fn load_and_play_current(&mut self, start_position: Duration) {
        if let Some(track) = self.queue.current_track().cloned() {
            match self.resolver.resolve_stream(&track) {
                Ok(stream) => {
                    let _ = self.worker_tx.send(PlayerCommand::LoadStream {
                        track,
                        stream,
                        start_position,
                    });
                }
                Err(e) => {
                    let err_msg = format!(
                        "Failed to resolve stream for track '{}': {}",
                        track.title, e
                    );
                    tracing::error!("{}", err_msg);
                    let _ = self.event_tx.send(SessionEvent::Error(err_msg));
                }
            }
        }
    }

    fn seek(&mut self, pos: Duration) {
        if let Some(track) = self.queue.current_track().cloned() {
            match self.resolver.resolve_stream(&track) {
                Ok(stream) => {
                    let _ = self.worker_tx.send(PlayerCommand::LoadStream {
                        track,
                        stream,
                        start_position: pos,
                    });
                }
                Err(_) => {
                    let _ = self.worker_tx.send(PlayerCommand::Seek(pos));
                }
            }
        }
    }

    fn advance_next(&mut self) {
        if self.queue.next(self.repeat_mode).is_some() {
            self.load_and_play_current(Duration::ZERO);
        } else {
            let _ = self.worker_tx.send(PlayerCommand::Stop);
            if let Ok(mut lock) = self.cached_state.write() {
                lock.playback.status = PlaybackStatus::Stopped;
                lock.playback.position_secs = 0.0;
            }
            self.sync_and_emit();
        }
    }

    fn handle_worker_event(&mut self, event: PlayerEvent) {
        match event {
            PlayerEvent::StateChanged(playback_state) => {
                let mut state = self.cached_state.read().unwrap().clone();
                state.playback = playback_state;
                state.current_track = self.queue.current_track().cloned();
                state.upcoming_tracks = self
                    .queue
                    .upcoming_tracks()
                    .into_iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                state.is_shuffle = self.queue.is_shuffle();
                state.repeat_mode = self.repeat_mode;

                *self.cached_state.write().unwrap() = state.clone();
                let _ = self.event_tx.send(SessionEvent::StateChanged(state));
            }
            PlayerEvent::VisualizerLevels(levels) => {
                let _ = self.event_tx.send(SessionEvent::VisualizerLevels(levels));
            }
            PlayerEvent::TrackEnded(_) => match self.repeat_mode {
                RepeatMode::One => {
                    self.load_and_play_current(Duration::ZERO);
                }
                _ => {
                    self.advance_next();
                }
            },
            PlayerEvent::Error(err) => {
                let _ = self.event_tx.send(SessionEvent::Error(err));
            }
        }
    }

    fn sync_and_emit(&mut self) {
        let mut state = self.cached_state.read().unwrap().clone();
        state.current_track = self.queue.current_track().cloned();
        state.upcoming_tracks = self
            .queue
            .upcoming_tracks()
            .into_iter()
            .map(|(_, t)| t.clone())
            .collect();
        state.is_shuffle = self.queue.is_shuffle();
        state.repeat_mode = self.repeat_mode;

        *self.cached_state.write().unwrap() = state.clone();
        let _ = self.event_tx.send(SessionEvent::StateChanged(state));
    }
}
