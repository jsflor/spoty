use crate::engine::{AudioEngine, PlayerCommand, PlayerEvent};
use crate::sink::AudioSink;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

pub struct AudioWorker {
    engine: AudioEngine,
    cmd_rx: Receiver<PlayerCommand>,
    event_tx: Sender<PlayerEvent>,
}

impl AudioWorker {
    pub fn spawn(
        cmd_rx: Receiver<PlayerCommand>,
        event_tx: Sender<PlayerEvent>,
    ) -> std::thread::JoinHandle<()> {
        Self::spawn_with_engine(cmd_rx, event_tx, AudioEngine::new())
    }

    pub fn spawn_with_sink(
        cmd_rx: Receiver<PlayerCommand>,
        event_tx: Sender<PlayerEvent>,
        sink: Box<dyn AudioSink>,
    ) -> std::thread::JoinHandle<()> {
        Self::spawn_with_engine(cmd_rx, event_tx, AudioEngine::new_with_sink(sink))
    }

    pub fn spawn_with_engine(
        cmd_rx: Receiver<PlayerCommand>,
        event_tx: Sender<PlayerEvent>,
        engine: AudioEngine,
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("spoty-audio-worker".into())
            .spawn(move || {
                let mut worker = AudioWorker {
                    engine,
                    cmd_rx,
                    event_tx,
                };
                worker.run();
            })
            .expect("Failed to spawn audio worker thread")
    }

    fn run(&mut self) {
        let mut last_tick = Instant::now();
        let tick_interval = Duration::from_millis(80); // ~12 fps level updates

        loop {
            match self.cmd_rx.recv_timeout(Duration::from_millis(40)) {
                Ok(cmd) => {
                    let events = self.engine.handle_command(cmd);
                    for ev in events {
                        let _ = self.event_tx.send(ev);
                    }
                    last_tick = Instant::now();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }

            let now = Instant::now();
            let elapsed = now.duration_since(last_tick);

            if elapsed >= tick_interval {
                let events = self.engine.tick(elapsed);
                for ev in events {
                    let _ = self.event_tx.send(ev);
                }
                last_tick = now;
            }
        }
    }
}
