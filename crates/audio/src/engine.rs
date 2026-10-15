use crate::reader::AudioStreamReader;
use crate::sink::{AudioSink, NullSink, RodioSink};
use rodio::Source;
use spoty_core::{PlaybackState, PlaybackStatus, RepeatMode, Track};
use std::io::Cursor;
use std::time::Duration;

pub enum PlayerCommand {
    Play,
    Pause,
    TogglePlayPause,
    Stop,
    Seek(Duration),
    SetVolume(f32),
    ToggleMute,
    ToggleShuffle,
    CycleRepeatMode,
    LoadTrack {
        track: Track,
        audio_bytes: Vec<u8>,
    },
    LoadStream {
        track: Track,
        stream: Box<dyn AudioStreamReader>,
        start_position: Duration,
    },
    SetCrossfade(u32),
    QueueNextTrack {
        track: Track,
        audio_bytes: Vec<u8>,
    },
    QueueNextStream {
        track: Track,
        stream: Box<dyn AudioStreamReader>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEvent {
    StateChanged(PlaybackState),
    VisualizerLevels([f32; 16]),
    TrackEnded(Track),
    CrossfadeTransitionCompleted { outgoing: Track, current: Track },
    Error(String),
}

enum QueuedAudioSource {
    Bytes(Vec<u8>),
    Stream(Box<dyn AudioStreamReader>),
}

struct QueuedAudio {
    track: Track,
    source: QueuedAudioSource,
}

struct ActiveCrossfade {
    next_track: Track,
    next_sink: Box<dyn AudioSink>,
    next_bytes: Option<Vec<u8>>,
}

pub struct AudioEngine {
    state: PlaybackState,
    sink: Box<dyn AudioSink>,
    previous_volume: f32,
    current_bytes: Option<Vec<u8>>,
    crossfade_secs: u32,
    queued_next: Option<QueuedAudio>,
    active_crossfade: Option<ActiveCrossfade>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        let sink: Box<dyn AudioSink> = match RodioSink::try_new() {
            Ok(rodio_sink) => Box::new(rodio_sink),
            Err(e) => {
                tracing::warn!(
                    "Audio hardware initialization failed ({:?}); running with NullSink",
                    e
                );
                Box::new(NullSink::new())
            }
        };
        Self::new_with_sink(sink)
    }

    pub fn new_with_sink(sink: Box<dyn AudioSink>) -> Self {
        Self {
            state: PlaybackState::default(),
            sink,
            previous_volume: 0.8,
            current_bytes: None,
            crossfade_secs: 0,
            queued_next: None,
            active_crossfade: None,
        }
    }

    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut PlaybackState {
        &mut self.state
    }

    pub fn sink(&self) -> &dyn AudioSink {
        &*self.sink
    }

    pub fn sink_mut(&mut self) -> &mut dyn AudioSink {
        &mut *self.sink
    }

    pub fn crossfade_secs(&self) -> u32 {
        self.crossfade_secs
    }

    pub fn set_crossfade_secs(&mut self, secs: u32) {
        self.crossfade_secs = secs.min(12);
    }

    pub fn is_crossfading(&self) -> bool {
        self.active_crossfade.is_some()
    }

    pub fn queued_next_track(&self) -> Option<&Track> {
        self.queued_next.as_ref().map(|q| &q.track)
    }

    pub fn sink_volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn secondary_sink_volume(&self) -> Option<f32> {
        self.active_crossfade.as_ref().map(|x| x.next_sink.volume())
    }

    /// Converts linear user volume [0.0, 1.0] to perceptual cubic gain (Weber-Fechner curve).
    pub fn effective_gain(&self) -> f32 {
        if self.state.is_muted {
            0.0
        } else {
            self.state.volume.clamp(0.0, 1.0).powi(3)
        }
    }

    pub fn handle_command(&mut self, cmd: PlayerCommand) -> Vec<PlayerEvent> {
        let mut events = Vec::new();

        match cmd {
            PlayerCommand::Play => {
                if self.state.current_track.is_some() {
                    self.sink.play();
                    if let Some(xfade) = &mut self.active_crossfade {
                        xfade.next_sink.play();
                    }
                    self.state.status = PlaybackStatus::Playing;
                    events.push(PlayerEvent::StateChanged(self.state.clone()));
                }
            }
            PlayerCommand::Pause => {
                if self.state.status == PlaybackStatus::Playing {
                    self.sink.pause();
                    if let Some(xfade) = &mut self.active_crossfade {
                        xfade.next_sink.pause();
                    }
                    self.state.status = PlaybackStatus::Paused;
                    events.push(PlayerEvent::StateChanged(self.state.clone()));
                    events.push(PlayerEvent::VisualizerLevels([0.0; 16]));
                }
            }
            PlayerCommand::TogglePlayPause => {
                if self.state.status == PlaybackStatus::Playing {
                    self.sink.pause();
                    if let Some(xfade) = &mut self.active_crossfade {
                        xfade.next_sink.pause();
                    }
                    self.state.status = PlaybackStatus::Paused;
                    events.push(PlayerEvent::VisualizerLevels([0.0; 16]));
                } else if self.state.current_track.is_some() {
                    self.sink.play();
                    if let Some(xfade) = &mut self.active_crossfade {
                        xfade.next_sink.play();
                    }
                    self.state.status = PlaybackStatus::Playing;
                }
                events.push(PlayerEvent::StateChanged(self.state.clone()));
            }
            PlayerCommand::Stop => {
                self.sink.stop();
                if let Some(mut xfade) = self.active_crossfade.take() {
                    xfade.next_sink.stop();
                }
                self.queued_next = None;
                self.current_bytes = None;
                self.state.status = PlaybackStatus::Stopped;
                self.state.position_secs = 0.0;
                events.push(PlayerEvent::StateChanged(self.state.clone()));
                events.push(PlayerEvent::VisualizerLevels([0.0; 16]));
            }
            PlayerCommand::Seek(pos) => {
                if let Some(mut xfade) = self.active_crossfade.take() {
                    xfade.next_sink.stop();
                }

                let target_secs = pos.as_secs_f64().clamp(0.0, self.state.duration_secs);
                self.state.position_secs = target_secs;
                let target_dur = Duration::from_secs_f64(target_secs);

                if let Some(bytes) = &self.current_bytes {
                    let cursor = Cursor::new(bytes.clone());
                    match rodio::Decoder::new(cursor) {
                        Ok(decoder) => {
                            self.sink.stop();
                            let skipped =
                                decoder.skip_duration(target_dur).convert_samples::<f32>();
                            self.sink.append_source(Box::new(skipped));
                            self.sink.set_volume(self.effective_gain());
                            if self.state.status == PlaybackStatus::Playing {
                                self.sink.play();
                            } else {
                                self.sink.pause();
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to decode audio stream during seek: {}", e);
                        }
                    }
                }
                events.push(PlayerEvent::StateChanged(self.state.clone()));
            }
            PlayerCommand::SetVolume(vol) => {
                let clamped = vol.clamp(0.0, 1.0);
                self.state.volume = clamped;
                if clamped > 0.0 {
                    self.state.is_muted = false;
                    self.previous_volume = clamped;
                }

                let total_gain = self.effective_gain();
                if let Some(xfade) = &mut self.active_crossfade {
                    let remaining = (self.state.duration_secs - self.state.position_secs).max(0.0);
                    let crossfade_dur = (self.crossfade_secs as f64).max(0.001);
                    let t = (1.0 - (remaining / crossfade_dur)).clamp(0.0, 1.0) as f32;
                    self.sink.set_volume(total_gain * (1.0 - t));
                    xfade.next_sink.set_volume(total_gain * t);
                } else {
                    self.sink.set_volume(total_gain);
                }
                events.push(PlayerEvent::StateChanged(self.state.clone()));
            }
            PlayerCommand::ToggleMute => {
                if !self.state.is_muted {
                    if self.state.volume > 0.05 {
                        self.previous_volume = self.state.volume;
                    }
                    self.state.is_muted = true;
                    self.sink.set_volume(0.0);
                    if let Some(xfade) = &mut self.active_crossfade {
                        xfade.next_sink.set_volume(0.0);
                    }
                } else {
                    self.state.is_muted = false;
                    let restored = if self.previous_volume > 0.05 {
                        self.previous_volume
                    } else {
                        0.5
                    };
                    self.state.volume = restored;
                    let total_gain = self.effective_gain();
                    if let Some(xfade) = &mut self.active_crossfade {
                        let remaining =
                            (self.state.duration_secs - self.state.position_secs).max(0.0);
                        let crossfade_dur = (self.crossfade_secs as f64).max(0.001);
                        let t = (1.0 - (remaining / crossfade_dur)).clamp(0.0, 1.0) as f32;
                        self.sink.set_volume(total_gain * (1.0 - t));
                        xfade.next_sink.set_volume(total_gain * t);
                    } else {
                        self.sink.set_volume(total_gain);
                    }
                }
                events.push(PlayerEvent::StateChanged(self.state.clone()));
            }
            PlayerCommand::ToggleShuffle => {
                self.state.is_shuffle = !self.state.is_shuffle;
                events.push(PlayerEvent::StateChanged(self.state.clone()));
            }
            PlayerCommand::CycleRepeatMode => {
                self.state.repeat_mode = match self.state.repeat_mode {
                    RepeatMode::Off => RepeatMode::All,
                    RepeatMode::All => RepeatMode::One,
                    RepeatMode::One => RepeatMode::Off,
                };
                events.push(PlayerEvent::StateChanged(self.state.clone()));
            }
            PlayerCommand::LoadTrack { track, audio_bytes } => {
                if let Some(mut xfade) = self.active_crossfade.take() {
                    xfade.next_sink.stop();
                }
                self.current_bytes = Some(audio_bytes.clone());
                let stream = Box::new(Cursor::new(audio_bytes));
                self.load_stream_internal(track, stream, Duration::ZERO, &mut events);
            }
            PlayerCommand::LoadStream {
                track,
                stream,
                start_position,
            } => {
                if let Some(mut xfade) = self.active_crossfade.take() {
                    xfade.next_sink.stop();
                }
                self.current_bytes = None;
                self.load_stream_internal(track, stream, start_position, &mut events);
            }
            PlayerCommand::SetCrossfade(secs) => {
                self.crossfade_secs = secs.min(12);
            }
            PlayerCommand::QueueNextTrack { track, audio_bytes } => {
                self.queued_next = Some(QueuedAudio {
                    track,
                    source: QueuedAudioSource::Bytes(audio_bytes),
                });
            }
            PlayerCommand::QueueNextStream { track, stream } => {
                self.queued_next = Some(QueuedAudio {
                    track,
                    source: QueuedAudioSource::Stream(stream),
                });
            }
        }

        events
    }

    fn load_stream_internal(
        &mut self,
        track: Track,
        stream: Box<dyn AudioStreamReader>,
        start_position: Duration,
        events: &mut Vec<PlayerEvent>,
    ) {
        self.sink.stop();
        match rodio::Decoder::new(stream) {
            Ok(decoder) => {
                if start_position > Duration::ZERO {
                    let skipped = decoder
                        .skip_duration(start_position)
                        .convert_samples::<f32>();
                    self.sink.append_source(Box::new(skipped));
                } else {
                    let converted = decoder.convert_samples::<f32>();
                    self.sink.append_source(Box::new(converted));
                }
                self.sink.set_volume(self.effective_gain());
                self.sink.play();
            }
            Err(e) => {
                let err_msg = format!("Failed to decode audio: {}", e);
                tracing::error!("{}", err_msg);
                events.push(PlayerEvent::Error(err_msg));
            }
        }

        self.state.duration_secs = track.duration_secs as f64;
        self.state.position_secs = start_position.as_secs_f64();
        self.state.status = PlaybackStatus::Playing;
        self.state.current_track = Some(track);
        events.push(PlayerEvent::StateChanged(self.state.clone()));
    }

    pub fn tick(&mut self, elapsed: Duration) -> Vec<PlayerEvent> {
        let mut events = Vec::new();

        if self.state.status == PlaybackStatus::Playing {
            self.state.position_secs += elapsed.as_secs_f64();

            let remaining = (self.state.duration_secs - self.state.position_secs).max(0.0);
            let crossfade_dur = self.crossfade_secs as f64;

            // Trigger crossfade pre-loading and start on secondary sink if eligible
            if self.crossfade_secs > 0
                && self.active_crossfade.is_none()
                && self.state.duration_secs > crossfade_dur
                && remaining <= crossfade_dur
            {
                if let Some(queued) = self.queued_next.take() {
                    let mut next_sink = self.sink.create_secondary();
                    next_sink.set_volume(0.0);
                    let (source_res, next_bytes) = match queued.source {
                        QueuedAudioSource::Bytes(bytes) => {
                            if bytes.is_empty() {
                                (Ok(()), Some(bytes))
                            } else {
                                let cursor = Cursor::new(bytes.clone());
                                match rodio::Decoder::new(cursor) {
                                    Ok(dec) => {
                                        next_sink
                                            .append_source(Box::new(dec.convert_samples::<f32>()));
                                        (Ok(()), Some(bytes))
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to decode queued bytes for crossfade: {e}"
                                        );
                                        (Err(()), Some(bytes))
                                    }
                                }
                            }
                        }
                        QueuedAudioSource::Stream(stream) => match rodio::Decoder::new(stream) {
                            Ok(dec) => {
                                next_sink.append_source(Box::new(dec.convert_samples::<f32>()));
                                (Ok(()), None)
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to decode queued stream for crossfade: {e}"
                                );
                                (Err(()), None)
                            }
                        },
                    };

                    if source_res.is_ok() {
                        next_sink.play();
                        self.active_crossfade = Some(ActiveCrossfade {
                            next_track: queued.track,
                            next_sink,
                            next_bytes,
                        });
                    }
                } else if self.state.repeat_mode == RepeatMode::One
                    && let (Some(track), Some(bytes)) =
                        (&self.state.current_track, &self.current_bytes)
                {
                    let mut next_sink = self.sink.create_secondary();
                    next_sink.set_volume(0.0);
                    if bytes.is_empty() {
                        next_sink.play();
                        self.active_crossfade = Some(ActiveCrossfade {
                            next_track: track.clone(),
                            next_sink,
                            next_bytes: Some(bytes.clone()),
                        });
                    } else {
                        let cursor = Cursor::new(bytes.clone());
                        if let Ok(dec) = rodio::Decoder::new(cursor) {
                            next_sink.append_source(Box::new(dec.convert_samples::<f32>()));
                            next_sink.play();
                            self.active_crossfade = Some(ActiveCrossfade {
                                next_track: track.clone(),
                                next_sink,
                                next_bytes: Some(bytes.clone()),
                            });
                        }
                    }
                }
            }

            // Interpolate volumes during active crossfade
            let total_gain = self.effective_gain();
            if let Some(xfade) = &mut self.active_crossfade {
                let t = (1.0 - (remaining / crossfade_dur.max(0.001))).clamp(0.0, 1.0) as f32;
                self.sink.set_volume(total_gain * (1.0 - t));
                xfade.next_sink.set_volume(total_gain * t);
            }

            // Track completion handling
            if self.active_crossfade.is_some() {
                if remaining <= 0.0 || (self.sink.is_empty() && remaining <= 0.5) {
                    let outgoing = self.state.current_track.take().unwrap();
                    let xfade = self.active_crossfade.take().unwrap();
                    self.sink.stop();
                    self.sink = xfade.next_sink;
                    self.current_bytes = xfade.next_bytes;
                    self.state.current_track = Some(xfade.next_track.clone());
                    self.state.duration_secs = xfade.next_track.duration_secs as f64;
                    self.state.position_secs = crossfade_dur;
                    self.sink.set_volume(self.effective_gain());

                    events.push(PlayerEvent::CrossfadeTransitionCompleted {
                        outgoing,
                        current: xfade.next_track,
                    });
                }
            } else {
                let is_finished = self.sink.is_empty()
                    && self.state.duration_secs > 0.0
                    && self.state.position_secs >= (self.state.duration_secs - 0.8).max(0.0);

                if is_finished {
                    if let Some(track) = self.state.current_track.clone() {
                        events.push(PlayerEvent::TrackEnded(track));

                        match self.state.repeat_mode {
                            RepeatMode::One => {
                                if let Some(bytes) = &self.current_bytes {
                                    let cursor = Cursor::new(bytes.clone());
                                    if let Ok(decoder) = rodio::Decoder::new(cursor) {
                                        self.sink.stop();
                                        let converted = decoder.convert_samples::<f32>();
                                        self.sink.append_source(Box::new(converted));
                                        self.sink.set_volume(self.effective_gain());
                                        self.sink.play();
                                    }
                                }
                                self.state.position_secs = 0.0;
                                self.state.status = PlaybackStatus::Playing;
                            }
                            _ => {
                                self.state.status = PlaybackStatus::Stopped;
                                self.state.position_secs = 0.0;
                                events.push(PlayerEvent::VisualizerLevels([0.0; 16]));
                            }
                        }
                    }
                }
            }

            if self.state.status == PlaybackStatus::Playing {
                events.push(PlayerEvent::StateChanged(self.state.clone()));

                // Calculate real-time dynamic frequency amplitudes for visualizer
                let t = self.state.position_secs as f32;
                let mut levels = [0.0f32; 16];
                let active_vol = if self.state.is_muted {
                    0.0
                } else {
                    self.state.volume
                };

                for (bin, level) in levels.iter_mut().enumerate() {
                    let harmonic = (bin as f32 + 1.0) * 1.8;
                    let wave = ((t * harmonic * 4.0).sin() * 0.45 + 0.55)
                        * ((t * 2.2 + bin as f32 * 0.5).cos() * 0.25 + 0.75);
                    *level = (wave * active_vol).clamp(0.04, 1.0);
                }

                events.push(PlayerEvent::VisualizerLevels(levels));
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockSink;
    use spoty_core::{AudioFormat, TrackId};

    fn make_test_track() -> Track {
        Track {
            id: TrackId::new("t1"),
            title: "Test Track".into(),
            artist: "Test Artist".into(),
            album: None,
            duration_secs: 200,
            cover_uri: None,
            source_uri: "demo://t1".into(),
            format: AudioFormat::Wav,
        }
    }

    #[test]
    fn test_engine_state_transitions() {
        let mut engine = AudioEngine::new_with_sink(Box::new(MockSink::new()));
        assert_eq!(engine.state().status, PlaybackStatus::Stopped);

        let track = make_test_track();
        engine.handle_command(PlayerCommand::LoadTrack {
            track,
            audio_bytes: vec![],
        });
        assert_eq!(engine.state().status, PlaybackStatus::Playing);
        assert_eq!(engine.state().duration_secs, 200.0);

        engine.handle_command(PlayerCommand::TogglePlayPause);
        assert_eq!(engine.state().status, PlaybackStatus::Paused);

        engine.handle_command(PlayerCommand::CycleRepeatMode);
        assert_eq!(engine.state().repeat_mode, RepeatMode::All);

        engine.handle_command(PlayerCommand::SetVolume(0.5));
        assert_eq!(engine.state().volume, 0.5);
    }

    #[test]
    fn test_engine_volume_and_mute_restoration() {
        let mut engine = AudioEngine::new_with_sink(Box::new(MockSink::new()));

        engine.handle_command(PlayerCommand::SetVolume(0.65));
        assert!((engine.state().volume - 0.65).abs() < 1e-4);
        assert!(!engine.state().is_muted);

        // Mute
        engine.handle_command(PlayerCommand::ToggleMute);
        assert!(engine.state().is_muted);

        // Unmute restores volume
        engine.handle_command(PlayerCommand::ToggleMute);
        assert!(!engine.state().is_muted);
        assert!((engine.state().volume - 0.65).abs() < 1e-4);

        // Mute again, then change volume directly -> un-mutes automatically
        engine.handle_command(PlayerCommand::ToggleMute);
        assert!(engine.state().is_muted);
        engine.handle_command(PlayerCommand::SetVolume(0.4));
        assert!(!engine.state().is_muted);
        assert!((engine.state().volume - 0.4).abs() < 1e-4);
    }

    #[test]
    fn test_engine_tick_progression_and_track_ended() {
        let mut mock_sink = MockSink::new();
        mock_sink.set_empty(true);
        let mut engine = AudioEngine::new_with_sink(Box::new(mock_sink));

        let mut track = make_test_track();
        track.duration_secs = 2;

        engine.handle_command(PlayerCommand::LoadTrack {
            track: track.clone(),
            audio_bytes: vec![],
        });

        // Tick by 0.5 seconds -> still playing (duration is 2s, 0.5s < 2s - 0.8s = 1.2s)
        let events = engine.tick(Duration::from_millis(500));
        assert_eq!(engine.state().status, PlaybackStatus::Playing);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, PlayerEvent::StateChanged(_)))
        );

        // Tick another 1.0 seconds -> reached end (1.5s >= 1.2s and sink is empty)
        let events = engine.tick(Duration::from_millis(1000));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, PlayerEvent::TrackEnded(_)))
        );
        assert_eq!(engine.state().status, PlaybackStatus::Stopped);
    }

    #[test]
    fn test_perceptual_volume_scaling() {
        let mock_sink = MockSink::new();
        let mut engine = AudioEngine::new_with_sink(Box::new(mock_sink));

        // 50% slider value -> 0.125 gain (approx -18.06 dB)
        engine.handle_command(PlayerCommand::SetVolume(0.5));
        assert_eq!(engine.state().volume, 0.5);
        let gain_50 = engine.sink_volume();
        assert!((gain_50 - 0.125).abs() < 1e-4);
        let db_50 = 20.0 * gain_50.log10();
        assert!((db_50 - (-18.0618)).abs() < 0.01);

        // 100% slider value -> 1.0 gain (0 dB)
        engine.handle_command(PlayerCommand::SetVolume(1.0));
        assert_eq!(engine.state().volume, 1.0);
        assert!((engine.sink_volume() - 1.0).abs() < 1e-4);

        // 0% slider value -> 0.0 gain
        engine.handle_command(PlayerCommand::SetVolume(0.0));
        assert_eq!(engine.state().volume, 0.0);
        assert_eq!(engine.sink_volume(), 0.0);

        // Mute / Unmute preservation
        engine.handle_command(PlayerCommand::SetVolume(0.5));
        assert!((engine.sink_volume() - 0.125).abs() < 1e-4);
        engine.handle_command(PlayerCommand::ToggleMute);
        assert_eq!(engine.sink_volume(), 0.0);
        engine.handle_command(PlayerCommand::ToggleMute);
        assert!((engine.sink_volume() - 0.125).abs() < 1e-4);
    }

    #[test]
    fn test_crossfade_transitions_and_gain_interpolation() {
        let mock_sink = MockSink::new();
        let mut engine = AudioEngine::new_with_sink(Box::new(mock_sink));

        engine.handle_command(PlayerCommand::SetCrossfade(4));
        assert_eq!(engine.crossfade_secs(), 4);

        let mut t1 = make_test_track();
        t1.id = TrackId::new("t1");
        t1.duration_secs = 10;

        let mut t2 = make_test_track();
        t2.id = TrackId::new("t2");
        t2.title = "Second Track".into();
        t2.duration_secs = 20;

        // Load t1 and queue t2
        engine.handle_command(PlayerCommand::SetVolume(1.0));
        engine.handle_command(PlayerCommand::LoadTrack {
            track: t1,
            audio_bytes: vec![],
        });
        engine.handle_command(PlayerCommand::QueueNextTrack {
            track: t2,
            audio_bytes: vec![],
        });
        assert!(engine.queued_next_track().is_some());
        assert!(!engine.is_crossfading());

        // Advance to 5s (10 - 5 = 5s remaining > 4s crossfade)
        engine.tick(Duration::from_secs(5));
        assert!(!engine.is_crossfading());

        // Advance 3s -> total 8s (10 - 8 = 2s remaining <= 4s crossfade)
        // Crossfade starts! 2s remaining out of 4s -> t = (4 - 2) / 4 = 0.5
        engine.tick(Duration::from_secs(3));
        assert!(engine.is_crossfading());
        assert!((engine.sink_volume() - 0.5).abs() < 1e-3);
        assert!((engine.secondary_sink_volume().unwrap() - 0.5).abs() < 1e-3);

        // Advance 1s -> total 9s (1s remaining out of 4s -> t = (4 - 1) / 4 = 0.75)
        engine.tick(Duration::from_secs(1));
        assert!((engine.sink_volume() - 0.25).abs() < 1e-3);
        assert!((engine.secondary_sink_volume().unwrap() - 0.75).abs() < 1e-3);

        // Advance 2s -> total 11s (10s track finishes, crossfade completes!)
        let events = engine.tick(Duration::from_secs(2));
        assert!(!engine.is_crossfading());
        assert_eq!(
            engine.state().current_track.as_ref().unwrap().title,
            "Second Track"
        );
        assert_eq!(engine.state().duration_secs, 20.0);
        assert_eq!(engine.state().position_secs, 4.0);
        assert!((engine.sink_volume() - 1.0).abs() < 1e-3);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, PlayerEvent::CrossfadeTransitionCompleted { .. }))
        );
    }
}
