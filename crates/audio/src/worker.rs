use crate::engine::{PlayerCommand, PlayerEvent};
use rodio::{Decoder, OutputStream, Sink};
use spoty_core::{PlaybackState, PlaybackStatus, RepeatMode};
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

pub struct AudioWorker {
    cmd_rx: Receiver<PlayerCommand>,
    event_tx: Sender<PlayerEvent>,
    state: PlaybackState,
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
                };
                worker.run();
            })
            .expect("Failed to spawn audio worker thread")
    }

    fn run(&mut self) {
        let stream_res = OutputStream::try_default();
        let (stream, sink_opt) = match stream_res {
            Ok((stream, stream_handle)) => {
                let sink = Sink::try_new(&stream_handle).ok();
                (Some(stream), sink)
            }
            Err(e) => {
                tracing::warn!(
                    "Audio hardware initialization failed ({:?}); running in headless playback mode",
                    e
                );
                (None, None)
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
                        if let Some(sink) = &sink_opt {
                            sink.stop();
                        }
                        self.state.status = PlaybackStatus::Stopped;
                        self.state.position_secs = 0.0;
                        self.emit_state();
                        let _ = self.event_tx.send(PlayerEvent::VisualizerLevels([0.0; 16]));
                    }
                    PlayerCommand::Seek(pos) => {
                        let target_secs = pos.as_secs_f64().clamp(0.0, self.state.duration_secs);
                        self.state.position_secs = target_secs;

                        if let Some(sink) = &sink_opt {
                            let _ = sink.try_seek(Duration::from_secs_f64(target_secs));
                        }
                        self.emit_state();
                    }
                    PlayerCommand::SetVolume(vol) => {
                        let clamped = vol.clamp(0.0, 1.0);
                        self.state.volume = clamped;
                        if let Some(sink) = &sink_opt {
                            sink.set_volume(if self.state.is_muted { 0.0 } else { clamped });
                        }
                        self.emit_state();
                    }
                    PlayerCommand::ToggleMute => {
                        self.state.is_muted = !self.state.is_muted;
                        if let Some(sink) = &sink_opt {
                            sink.set_volume(if self.state.is_muted {
                                0.0
                            } else {
                                self.state.volume
                            });
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
                        if let Some(sink) = &sink_opt {
                            sink.stop();
                            let cursor = Cursor::new(audio_bytes.clone());
                            match Decoder::new(cursor) {
                                Ok(decoder) => {
                                    sink.append(decoder);
                                    sink.set_volume(if self.state.is_muted {
                                        0.0
                                    } else {
                                        self.state.volume
                                    });
                                    sink.play();
                                }
                                Err(e) => {
                                    let err_msg = format!("Failed to decode audio: {}", e);
                                    tracing::error!("{}", err_msg);
                                    let _ = self.event_tx.send(PlayerEvent::Error(err_msg));
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
                } else {
                    self.state.position_secs >= self.state.duration_secs
                };

                if is_finished {
                    if let Some(track) = self.state.current_track.clone() {
                        let _ = self.event_tx.send(PlayerEvent::TrackEnded(track.clone()));

                        match self.state.repeat_mode {
                            RepeatMode::One => {
                                if let Some(bytes) = &current_bytes {
                                    if let Some(sink) = &sink_opt {
                                        let cursor = Cursor::new(bytes.clone());
                                        if let Ok(decoder) = Decoder::new(cursor) {
                                            sink.append(decoder);
                                            sink.play();
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
