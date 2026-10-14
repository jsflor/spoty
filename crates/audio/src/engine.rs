use spoty_core::{PlaybackState, PlaybackStatus, RepeatMode, Track};
use std::time::Duration;

#[derive(Debug, Clone)]
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
    LoadTrack { track: Track, audio_bytes: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEvent {
    StateChanged(PlaybackState),
    VisualizerLevels([f32; 16]),
    TrackEnded(Track),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct AudioEngine {
    state: PlaybackState,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::default(),
        }
    }

    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    pub fn handle_command(&mut self, cmd: PlayerCommand) {
        match cmd {
            PlayerCommand::Play => {
                if self.state.current_track.is_some() {
                    self.state.status = PlaybackStatus::Playing;
                }
            }
            PlayerCommand::Pause => {
                if self.state.status == PlaybackStatus::Playing {
                    self.state.status = PlaybackStatus::Paused;
                }
            }
            PlayerCommand::TogglePlayPause => {
                self.state.status = match self.state.status {
                    PlaybackStatus::Playing => PlaybackStatus::Paused,
                    PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                        if self.state.current_track.is_some() {
                            PlaybackStatus::Playing
                        } else {
                            PlaybackStatus::Stopped
                        }
                    }
                    PlaybackStatus::Buffering => PlaybackStatus::Paused,
                };
            }
            PlayerCommand::Stop => {
                self.state.status = PlaybackStatus::Stopped;
                self.state.position_secs = 0.0;
            }
            PlayerCommand::Seek(pos) => {
                self.state.position_secs = pos.as_secs_f64();
            }
            PlayerCommand::SetVolume(vol) => {
                self.state.volume = vol.clamp(0.0, 1.0);
            }
            PlayerCommand::ToggleMute => {
                self.state.is_muted = !self.state.is_muted;
            }
            PlayerCommand::ToggleShuffle => {
                self.state.is_shuffle = !self.state.is_shuffle;
            }
            PlayerCommand::CycleRepeatMode => {
                self.state.repeat_mode = match self.state.repeat_mode {
                    RepeatMode::Off => RepeatMode::All,
                    RepeatMode::All => RepeatMode::One,
                    RepeatMode::One => RepeatMode::Off,
                };
            }
            PlayerCommand::LoadTrack { track, .. } => {
                self.state.duration_secs = track.duration_secs as f64;
                self.state.position_secs = 0.0;
                self.state.status = PlaybackStatus::Playing;
                self.state.current_track = Some(track);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spoty_core::{AudioFormat, TrackId};

    #[test]
    fn test_engine_state_transitions() {
        let mut engine = AudioEngine::new();
        assert_eq!(engine.state().status, PlaybackStatus::Stopped);

        let track = Track {
            id: TrackId::new("t1"),
            title: "Test Track".into(),
            artist: "Test Artist".into(),
            album: None,
            duration_secs: 200,
            cover_uri: None,
            source_uri: "demo://t1".into(),
            format: AudioFormat::Wav,
        };

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
}
