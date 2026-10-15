use crate::engine::{PlayerCommand, PlayerEvent};
use crate::worker::AudioWorker;
use spoty_core::{PlaybackState, Track};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Clone)]
pub struct PlayerController {
    cmd_tx: Sender<PlayerCommand>,
    cached_state: Arc<RwLock<PlaybackState>>,
}

impl PlayerController {
    pub fn new() -> (Self, Receiver<PlayerEvent>) {
        let (cmd_tx, cmd_rx) = channel();
        let (event_tx, event_rx) = channel();

        AudioWorker::spawn(cmd_rx, event_tx);

        let cached_state = Arc::new(RwLock::new(PlaybackState::default()));

        let controller = Self {
            cmd_tx,
            cached_state,
        };

        (controller, event_rx)
    }

    pub fn state(&self) -> PlaybackState {
        self.cached_state.read().unwrap().clone()
    }

    pub fn update_cached_state(&self, state: PlaybackState) {
        if let Ok(mut lock) = self.cached_state.write() {
            *lock = state;
        }
    }

    pub fn play(&self) {
        let _ = self.cmd_tx.send(PlayerCommand::Play);
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(PlayerCommand::Pause);
    }

    pub fn toggle_play_pause(&self) {
        let _ = self.cmd_tx.send(PlayerCommand::TogglePlayPause);
    }

    pub fn stop(&self) {
        let _ = self.cmd_tx.send(PlayerCommand::Stop);
    }

    pub fn seek(&self, position: Duration) {
        if let Ok(mut lock) = self.cached_state.write() {
            lock.position_secs = position.as_secs_f64();
        }
        let _ = self.cmd_tx.send(PlayerCommand::Seek(position));
    }

    pub fn set_volume(&self, volume: f32) {
        let _ = self.cmd_tx.send(PlayerCommand::SetVolume(volume));
    }

    pub fn toggle_mute(&self) {
        let _ = self.cmd_tx.send(PlayerCommand::ToggleMute);
    }

    pub fn toggle_shuffle(&self) {
        let _ = self.cmd_tx.send(PlayerCommand::ToggleShuffle);
    }

    pub fn cycle_repeat_mode(&self) {
        let _ = self.cmd_tx.send(PlayerCommand::CycleRepeatMode);
    }

    pub fn load_track(&self, track: Track, audio_bytes: Vec<u8>) {
        let _ = self
            .cmd_tx
            .send(PlayerCommand::LoadTrack { track, audio_bytes });
    }
}
