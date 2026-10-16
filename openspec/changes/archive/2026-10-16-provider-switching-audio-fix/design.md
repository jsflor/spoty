# Design: provider-switching-audio-fix

## 1. Context and Problem Statement

When the user switches storage providers (e.g. from Demo to Local Filesystem or back), Spoty performs three concurrent actions:
1. Replaces `self.provider` and registers a new `AppAudioResolver` on `self.session`.
2. Spawns an asynchronous background task on `tokio_rt` to run `catalog.sync_provider(prov)`.
3. Reads `catalog.library().tracks` synchronously to call `session.set_queue(tracks, None)`.

Because step 2 is asynchronous, step 3 executes before the new provider has fetched its library. Thus, the queue receives the old provider's tracks. When step 2 finishes, it updates the catalog in memory but does not update `self.session`. Furthermore, when a user clicks a song from the new provider, `SpotyApp::play_track` sends `session.play_track(track.id)`. Since `track.id` is not in the session queue, `SessionCoordinator` executes `load_and_play_current` with the previous provider's active track, passing it to the new provider's resolver, which fails.

## 2. Technical Architecture & Component Changes

```text
[User selects new Provider]
         │
         ▼
[SpotyApp::switch_provider]
   ├── 1. session.pause() & session.set_queue([], None)
   ├── 2. session.set_resolver(new_resolver)
   └── 3. tokio_rt.spawn(async move {
             catalog.sync_provider(provider).await;
             session.set_queue(catalog.library().tracks, None);
          })

[User clicks Track in Songs / View]
         │
         ▼
[SpotyApp::play_track]
   ├── Track in active queue? ──► session.play_track(track.id)
   └── Track not in queue?    ──► session.play_all(catalog.library().tracks, pos)

[SessionCoordinator::handle_session_command(PlayTrack(id))]
   ├── id in queue? ──► jump_to(pos) & load_and_play_current()
   └── id missing?  ──► log warning (do NOT play leftover track)
```

### 2.1 `crates/audio/src/session.rs`

In `SessionCoordinator::handle_session_command`:
```rust
SessionCommand::PlayTrack(track_id) => {
    if let Some(pos) = self.queue.tracks().iter().position(|t| t.id == track_id) {
        self.queue.jump_to(pos);
        self.load_and_play_current(Duration::ZERO);
    } else {
        tracing::warn!("PlayTrack ignored: track {} not found in session queue", track_id);
    }
}
```
This guarantees that requesting a missing `TrackId` never triggers playback of an arbitrary prior track.

### 2.2 `crates/ui/src/app.rs`

In `SpotyApp::switch_provider`:
```rust
fn switch_provider(&mut self, provider: Arc<dyn MusicProvider>, p_type: ProviderType) {
    self.session.pause();
    self.session.set_queue(Vec::new(), None);

    self.provider = Arc::clone(&provider);
    self.config.active_provider = p_type;
    let _ = self.config.save();
    self.nav.push(Route::Home);

    let new_resolver = Arc::new(AppAudioResolver {
        provider: Arc::clone(&provider),
        tokio_rt: Arc::clone(&self.tokio_rt),
    });
    self.session.set_resolver(new_resolver);

    let catalog = Arc::clone(&self.catalog);
    let prov = Arc::clone(&provider);
    let session = self.session.clone();
    self.tokio_rt.spawn(async move {
        match catalog.sync_provider(prov.as_ref()).await {
            Ok(()) => {
                let tracks = catalog.library().tracks;
                session.set_queue(tracks, None);
            }
            Err(e) => {
                tracing::error!("Failed to sync provider: {e}");
            }
        }
    });
}
```

In `SpotyApp::play_track`:
```rust
fn play_track(&mut self, track: Track) {
    let state = self.session.state();
    let in_queue = state.current_track.as_ref().map(|t| t.id == track.id).unwrap_or(false)
        || state.upcoming_tracks.iter().any(|t| t.id == track.id);

    if in_queue {
        self.session.play_track(track.id);
    } else {
        let tracks = self.catalog.library().tracks;
        if let Some(pos) = tracks.iter().position(|t| t.id == track.id) {
            self.session.play_all(tracks, pos);
        } else {
            self.session.play_all(vec![track], 0);
        }
    }
}
```

## 3. Invariants & Safety

- **RAM Constraint:** No audio buffering in UI memory; streams remain bounded and decoupled.
- **Thread Boundary:** Main UI loop never blocks on filesystem/network scans.
- **Provider Isolation:** Old provider streams cannot leak into new provider resolvers.
