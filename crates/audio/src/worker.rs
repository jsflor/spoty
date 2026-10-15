use crate::engine::{PlayerCommand, PlayerEvent};
use rodio::{Decoder, OutputStream, Sink, Source};
use spoty_core::{PlaybackState, PlaybackStatus, RepeatMode};
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

pub struct AudioWorker {
    cmd_rx: Receiver<PlayerCommand>,
    event_tx: Sender<PlayerEvent>,
    state: PlaybackState,
    previous_volume: f32,
}

impl AudioWorker {
    pub fn spawn(
        cmd_rx: Receiver<PlayerCommand>,
        event_tx: Sender<PlayerEvent>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("spoty-audio-worker".into())
            .spawn(move || {
                let mut worker = AudioWorker {
                    cmd_rx,
                    event_tx,
                    state: PlaybackState::default(),
                    previous_volume: 0.8,
                };
                worker.run();
            })
            .expect("Failed to spawn audio worker thread")
    }

    fn run(&mut self) {
        let stream_res = OutputStream::try_default();
        let (stream, stream_handle, mut sink_opt) = match stream_res {
            Ok((stream, stream_handle)) => {
                let sink = Sink::try_new(&stream_handle).ok();
                (Some(stream), Some(stream_handle), sink)
            }
            Err(e) => {
                tracing::warn!(
                    "Audio hardware initialization failed ({:?}); running in headless playback mode",
                    e
                );
                (None, None, None)
            }
        };

        let mut current_bytes: Option<Vec<u8>> = None;
        let mut last_tick = Instant::now();
        let tick_interval = Duration::from_millis(80); // ~12 fps level updates

        loop {
            match self.cmd_rx.recv_timeout(Duration::from_millis(40)) {
                Ok(cmd) => match cmd {
                    PlayerCommand::Play => {
                        if self.state.current_track.is_some() {
                            if let Some(sink) = &sink_opt {
                                sink.play();
                            }
                            self.state.status = PlaybackStatus::Playing;
                            self.emit_state();
                        }
                    }
                    PlayerCommand::Pause => {
                        if self.state.status == PlaybackStatus::Playing {
                            if let Some(sink) = &sink_opt {
                                sink.pause();
                            }
                            self.state.status = PlaybackStatus::Paused;
                            self.emit_state();
                            let _ = self.event_tx.send(PlayerEvent::VisualizerLevels([0.0; 16]));
                        }
                    }
                    PlayerCommand::TogglePlayPause => {
                        if self.state.status == PlaybackStatus::Playing {
                            if let Some(sink) = &sink_opt {
                                sink.pause();
                            }
                            self.state.status = PlaybackStatus::Paused;
                            let _ = self.event_tx.send(PlayerEvent::VisualizerLevels([0.0; 16]));
                        } else if self.state.current_track.is_some() {
                            if let Some(sink) = &sink_opt {
                                sink.play();
                            }
                            self.state.status = PlaybackStatus::Playing;
                        }
                        self.emit_state();
                    }
                    PlayerCommand::Stop => {
                        sink_opt = None;
                        if let Some(handle) = &stream_handle {
                            sink_opt = Sink::try_new(handle).ok();
                        }
                        self.state.status = PlaybackStatus::Stopped;
                        self.state.position_secs = 0.0;
                        self.emit_state();
                        let _ = self.event_tx.send(PlayerEvent::VisualizerLevels([0.0; 16]));
                    }
                    PlayerCommand::Seek(pos) => {
                        let target_secs = pos.as_secs_f64().clamp(0.0, self.state.duration_secs);
                        self.state.position_secs = target_secs;

                        if let (Some(handle), Some(bytes)) = (&stream_handle, &current_bytes) {
                            match Sink::try_new(handle) {
                                Ok(new_sink) => {
                                    let cursor = Cursor::new(bytes.clone());
                                    match Decoder::new(cursor) {
                                        Ok(decoder) => {
                                            let target_dur = Duration::from_secs_f64(target_secs);
                                            let skipped = decoder.skip_duration(target_dur);
                                            new_sink.append(skipped);
                                            new_sink.set_volume(if self.state.is_muted {
                                                0.0
                                            } else {
                                                self.state.volume
                                            });
                                            if self.state.status == PlaybackStatus::Playing {
                                                new_sink.play();
                                            } else {
                                                new_sink.pause();
                                            }
                                            sink_opt = Some(new_sink);
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "Failed to decode audio stream during seek: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to create audio sink for seek: {}", e);
                                }
                            }
                        }
                        last_tick = Instant::now();
                        self.emit_state();
                    }
                    PlayerCommand::SetVolume(vol) => {
                        let clamped = vol.clamp(0.0, 1.0);
                        self.state.volume = clamped;
                        if clamped > 0.0 {
                            self.state.is_muted = false;
                            self.previous_volume = clamped;
                        }
                        if let Some(sink) = &sink_opt {
                            sink.set_volume(if self.state.is_muted { 0.0 } else { clamped });
                        }
                        self.emit_state();
                    }
                    PlayerCommand::ToggleMute => {
                        if !self.state.is_muted {
                            // Muting
                            if self.state.volume > 0.05 {
                                self.previous_volume = self.state.volume;
                            }
                            self.state.is_muted = true;
                            if let Some(sink) = &sink_opt {
                                sink.set_volume(0.0);
                            }
                        } else {
                            // Unmuting
                            self.state.is_muted = false;
                            let restored = if self.previous_volume > 0.05 {
                                self.previous_volume
                            } else {
                                0.5
                            };
                            self.state.volume = restored;
                            if let Some(sink) = &sink_opt {
                                sink.set_volume(restored);
                            }
                        }
                        self.emit_state();
                    }
                    PlayerCommand::ToggleShuffle => {
                        self.state.is_shuffle = !self.state.is_shuffle;
                        self.emit_state();
                    }
                    PlayerCommand::CycleRepeatMode => {
                        self.state.repeat_mode = match self.state.repeat_mode {
                            RepeatMode::Off => RepeatMode::All,
                            RepeatMode::All => RepeatMode::One,
                            RepeatMode::One => RepeatMode::Off,
                        };
                        self.emit_state();
                    }
                    PlayerCommand::LoadTrack { track, audio_bytes } => {
                        if let Some(handle) = &stream_handle {
                            match Sink::try_new(handle) {
                                Ok(new_sink) => {
                                    let cursor = Cursor::new(audio_bytes.clone());
                                    match Decoder::new(cursor) {
                                        Ok(decoder) => {
                                            new_sink.append(decoder);
                                            new_sink.set_volume(if self.state.is_muted {
                                                0.0
                                            } else {
                                                self.state.volume
                                            });
                                            new_sink.play();
                                            sink_opt = Some(new_sink);
                                        }
                                        Err(e) => {
                                            let err_msg = format!("Failed to decode audio: {}", e);
                                            tracing::error!("{}", err_msg);
                                            let _ = self.event_tx.send(PlayerEvent::Error(err_msg));
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to create audio sink: {}", e);
                                }
                            }
                        }

                        current_bytes = Some(audio_bytes);
                        self.state.duration_secs = track.duration_secs as f64;
                        self.state.position_secs = 0.0;
                        self.state.status = PlaybackStatus::Playing;
                        self.state.current_track = Some(track);
                        last_tick = Instant::now();
                        self.emit_state();
                    }
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }

            let now = Instant::now();
            let elapsed = now.duration_since(last_tick);

            if self.state.status == PlaybackStatus::Playing {
                self.state.position_secs += elapsed.as_secs_f64();

                let is_finished = if let Some(sink) = &sink_opt {
                    sink.empty()
                        && self.state.duration_secs > 0.0
                        && self.state.position_secs >= (self.state.duration_secs - 0.8)
                } else {
                    self.state.duration_secs > 0.0
                        && self.state.position_secs >= self.state.duration_secs
                };

                if is_finished {
                    if let Some(track) = self.state.current_track.clone() {
                        let _ = self.event_tx.send(PlayerEvent::TrackEnded(track.clone()));

                        match self.state.repeat_mode {
                            RepeatMode::One => {
                                if let (Some(handle), Some(bytes)) =
                                    (&stream_handle, &current_bytes)
                                {
                                    if let Ok(new_sink) = Sink::try_new(handle) {
                                        let cursor = Cursor::new(bytes.clone());
                                        if let Ok(decoder) = Decoder::new(cursor) {
                                            new_sink.append(decoder);
                                            new_sink.set_volume(if self.state.is_muted {
                                                0.0
                                            } else {
                                                self.state.volume
                                            });
                                            new_sink.play();
                                            sink_opt = Some(new_sink);
                                        }
                                    }
                                    self.state.position_secs = 0.0;
                                    self.state.status = PlaybackStatus::Playing;
                                }
                            }
                            _ => {
                                self.state.status = PlaybackStatus::Stopped;
                                self.state.position_secs = 0.0;
                                let _ =
                                    self.event_tx.send(PlayerEvent::VisualizerLevels([0.0; 16]));
                            }
                        }
                    }
                }

                if elapsed >= tick_interval {
                    self.emit_state();

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

                    let _ = self.event_tx.send(PlayerEvent::VisualizerLevels(levels));
                    last_tick = now;
                }
            } else {
                last_tick = now;
            }
        }

        drop(stream);
    }

    fn emit_state(&self) {
        let _ = self
            .event_tx
            .send(PlayerEvent::StateChanged(self.state.clone()));
    }
}
