# Design: playback-session-pipeline

## Architectural Overview

```text
┌────────────────────────────────────────────────────────┐
│                   crates/ui (Shell)                    │
│                                                        │
│   SpotyApp ──► session.play_track(track_id)            │
│            ──► session.next() / session.prev()         │
│            ──► session.toggle_shuffle()                │
│            ◄── session_events (StateChanged)           │
└───────────────────────────┬────────────────────────────┘
                            │ Non-blocking Seam
                            ▼
┌────────────────────────────────────────────────────────┐
│            crates/audio (PlaybackSession)              │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                Session Coordinator               │  │
│  │  - Holds Queue (deterministic core model)        │  │
│  │  - Single authoritative SessionState             │  │
│  │  - Listens to Worker stream status               │  │
│  └───────────┬──────────────────────────┬───────────┘  │
│              │ Stream Request           │ Audio Sinks  │
│              ▼                          ▼              │
│  ┌───────────────────────┐  ┌───────────────────────┐  │
│  │   Track Stream Feeder │  │   AudioWorker Thread  │  │
│  │   - Pulls stream      │  │   - Symphonia decoder │  │
│  │   - Bounded ringbuf   │  │   - Rodio output sink │  │
│  │   - Stream-to-Sink    │  │   - EOF detection     │  │
│  └───────────────────────┘  └───────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

## Seam & Interface Definition

### `PlaybackSession` Interface

```rust
pub struct PlaybackSession {
    cmd_tx: std::sync::mpsc::Sender<SessionCommand>,
    state: std::sync::Arc<std::sync::RwLock<SessionState>>,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub playback: PlaybackState,
    pub current_track: Option<Track>,
    pub upcoming_tracks: Vec<Track>,
    pub is_shuffle: bool,
    pub repeat_mode: RepeatMode,
}

#[derive(Debug)]
pub enum SessionCommand {
    PlayTrack(TrackId),
    PlayAll { tracks: Vec<Track>, start_index: usize },
    TogglePlayPause,
    Play,
    Pause,
    Next,
    Prev,
    Seek(std::time::Duration),
    SetVolume(f32),
    ToggleMute,
    ToggleShuffle,
    CycleRepeatMode,
    SetQueue(Vec<Track>, Option<usize>),
    EnqueueNext(Track),
    EnqueueLast(Track),
    RemoveFromQueue(usize),
}
```

## Key Technical Decisions

### 1. Removing UI Memory Buffering
Currently, `SpotyApp` runs:
```rust
let mut bytes = Vec::new();
stream.read_to_end(&mut bytes).await;
controller.load_track(track, bytes);
```
Under this design, `PlaybackSession` is initialized with a track audio provider adapter or accepts an asynchronous stream reader handle. The stream is decoded directly into chunks on the background audio thread or streamed through a bounded ring buffer, ensuring RAM consumption remains strictly `<50MB` even during multi-hour high-resolution audio sessions.

### 2. Autonomous Queue Advancement
When `AudioWorker` detects track exhaustion (`is_finished`):
- Rather than emitting `TrackEnded` across to the UI, the worker notifies the session coordinator internally.
- The session coordinator queries `Queue::next(repeat_mode)`.
- If a subsequent track exists, it immediately begins streaming the next track.
- If queue is exhausted and repeat is off, state transitions to `Stopped`.
- A single consolidated `StateChanged(SessionState)` event is sent to the UI.

### 3. Unified State Machine
- `Queue` remains in `spoty-core` as a pure, zero-I/O domain model.
- `PlaybackSession` owns the active `Queue` instance and merges queue properties (current index, upcoming tracks, shuffle flag) into `SessionState`.
- Callers read state through `session.state()`, guaranteeing that `state.is_shuffle`, `state.current_track`, and `state.playback.status` can never get out of phase.

## Error Handling & Fallbacks

- If track stream acquisition fails (e.g., remote network drop or S3 timeout), `PlaybackSession` emits `SessionEvent::Error(err)` and attempts fallback or advances to the next playable track if configured.
- UI stays completely responsive: all stream resolutions occur on Tokio tasks owned by the session coordinator, never blocking egui frames.
