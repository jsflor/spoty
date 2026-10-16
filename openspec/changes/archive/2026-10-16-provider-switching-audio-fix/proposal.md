# Proposal: provider-switching-audio-fix

## Why

When a user switches music sources (for example, switching from Demo Music to a Local Folder, or vice versa) and selects a song from the new source, the playback system fails to play the selected song:

1. **Premature Queue Population & Out-of-Sync Session Queue (`crates/ui/src/app.rs`)**:
   `switch_provider` initiates `catalog.sync_provider` in a background Tokio task. Immediately following this, it calls `catalog.library().tracks` synchronously to initialize the `PlaybackSession` queue. Because the background sync has not completed, the queue is loaded with tracks from the **previous** provider. Furthermore, when `catalog.sync_provider` eventually finishes, it updates the catalog in memory but never updates `PlaybackSession`. As a result, the playback session queue remains permanently desynchronized from the active catalog.
2. **Erroneous Fallback Playback in `SessionCoordinator` (`crates/audio/src/session.rs`)**:
   In `SessionCoordinator::handle_session_command`, `SessionCommand::PlayTrack(track_id)` attempts to locate `track_id` in `self.queue`. If not found, it skips `self.queue.jump_to(pos)` but still proceeds to call `self.load_and_play_current(Duration::ZERO)`. This attempts to play whatever track was previously active in the queue (from the old provider).
3. **Audio Stream Resolver Failure**:
   Because `PlaybackSession` was updated with the new provider's `AudioStreamResolver`, attempting to play the old provider's track through the new resolver causes an immediate resolution error (e.g., `"Track not found in provider path map"` or `"Track not found in demo provider"`), breaking playback entirely.

## What Changes

1. **Asynchronous Queue Update on Provider Switch (`crates/ui/src/app.rs`)**:
   - Immediately pause any existing playback (`session.pause()`) and clear the existing queue (`session.set_queue(Vec::new(), None)`).
   - In the background Tokio task, once `catalog.sync_provider` succeeds, update `session` with the newly synced tracks (`session.set_queue(tracks, None)`).
   - Remove the premature synchronous queue assignment from `switch_provider`.
2. **Resilient Track Playback in `SpotyApp::play_track` (`crates/ui/src/app.rs`)**:
   - If the requested track is already present in the active queue, dispatch `session.play_track(track.id)`.
   - If the track is not present in the active queue, populate the queue with catalog tracks starting at the selected track (`session.play_all(tracks, pos)`), or play the individual track (`session.play_all(vec![track], 0)`).
3. **Guarded Playback in `SessionCoordinator` (`crates/audio/src/session.rs`)**:
   - Only call `self.load_and_play_current` in `SessionCommand::PlayTrack` if the track ID was found in the queue. If not found, log a warning and do not stream an arbitrary leftover track.
4. **Integration and Unit Tests**:
   - Add integration tests covering source switching and subsequent playback across providers.
   - Add unit test verifying that `SessionCommand::PlayTrack` ignores unknown track IDs.

## Capabilities

### Modified Capabilities
- `audio/playback-session`: `SessionCommand::PlayTrack` ensures tracks are only loaded when present in the queue.
- `ui/navigation-and-views`: Track selection handles queue replenishment when a track from outside the active queue is selected.

### New Capabilities
- `ui/provider-switching`: Seamless switching between storage providers with automatic session queue synchronization and clean playback reset.

## Impact

- **Crates Affected:**
  - `crates/audio`: Guard `SessionCommand::PlayTrack` against loading unmapped current tracks.
  - `crates/ui`: Update `switch_provider` and `play_track` to synchronize session state correctly during and after provider transitions.
- **Breaking Changes:** None. All public interfaces remain compatible.
