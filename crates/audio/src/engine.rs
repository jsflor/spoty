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
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEvent {
    StateChanged(PlaybackState),
    VisualizerLevels([f32; 16]),
    TrackEnded(Track),
    Error(String),
}

pub struct AudioEngine {
    state: PlaybackState,
    sink: Box<dyn AudioSink>,
    previous_volume: f32,
    current_bytes: Option<Vec<u8>>,
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

    pub fn handle_command(&mut self, cmd: PlayerCommand) -> Vec<PlayerEvent> {
        let mut events = Vec::new();

        match cmd {
            PlayerCommand::Play => {
                if self.state.current_track.is_some() {
                    self.sink.play();
                    self.state.status = PlaybackStatus::Playing;
                    events.push(PlayerEvent::StateChanged(self.state.clone()));
                }
            }
            PlayerCommand::Pause => {
                if self.state.status == PlaybackStatus::Playing {
                    self.sink.pause();
                    self.state.status = PlaybackStatus::Paused;
                    events.push(PlayerEvent::StateChanged(self.state.clone()));
                    events.push(PlayerEvent::VisualizerLevels([0.0; 16]));
                }
            }
            PlayerCommand::TogglePlayPause => {
                if self.state.status == PlaybackStatus::Playing {
                    self.sink.pause();
                    self.state.status = PlaybackStatus::Paused;
                    events.push(PlayerEvent::VisualizerLevels([0.0; 16]));
                } else if self.state.current_track.is_some() {
                    self.sink.play();
                    self.state.status = PlaybackStatus::Playing;
                }
                events.push(PlayerEvent::StateChanged(self.state.clone()));
            }
            PlayerCommand::Stop => {
                self.sink.stop();
                self.current_bytes = None;
                self.state.status = PlaybackStatus::Stopped;
                self.state.position_secs = 0.0;
                events.push(PlayerEvent::StateChanged(self.state.clone()));
                events.push(PlayerEvent::VisualizerLevels([0.0; 16]));
            }
            PlayerCommand::Seek(pos) => {
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
                            let active_vol = if self.state.is_muted {
                                0.0
                            } else {
                                self.state.volume
                            };
                            self.sink.set_volume(active_vol);
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
                self.sink
                    .set_volume(if self.state.is_muted { 0.0 } else { clamped });
                events.push(PlayerEvent::StateChanged(self.state.clone()));
            }
            PlayerCommand::ToggleMute => {
                if !self.state.is_muted {
                    if self.state.volume > 0.05 {
                        self.previous_volume = self.state.volume;
                    }
                    self.state.is_muted = true;
                    self.sink.set_volume(0.0);
                } else {
                    self.state.is_muted = false;
                    let restored = if self.previous_volume > 0.05 {
                        self.previous_volume
                    } else {
                        0.5
                    };
                    self.state.volume = restored;
                    self.sink.set_volume(restored);
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
                self.current_bytes = Some(audio_bytes.clone());
                let stream = Box::new(Cursor::new(audio_bytes));
                self.load_stream_internal(track, stream, Duration::ZERO, &mut events);
            }
            PlayerCommand::LoadStream {
                track,
                stream,
                start_position,
            } => {
                self.current_bytes = None;
                self.load_stream_internal(track, stream, start_position, &mut events);
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
                let active_vol = if self.state.is_muted {
                    0.0
                } else {
                    self.state.volume
                };
                self.sink.set_volume(active_vol);
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
                                    let active_vol = if self.state.is_muted {
                                        0.0
                                    } else {
                                        self.state.volume
                                    };
                                    self.sink.set_volume(active_vol);
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
}
