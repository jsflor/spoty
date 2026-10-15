# Design: audio-sink-adapter-seam

## Architectural Overview

```text
┌────────────────────────────────────────────────────────┐
│                   crates/audio Core                    │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                    AudioEngine                   │  │
│  │  - Deterministic playback state transitions      │  │
│  │  - Volume attenuation & mute restoration logic   │  │
│  │  - Seek timestamp calculation                    │  │
│  │  - Decoder source management                     │  │
│  └──────────────────────────┬───────────────────────┘  │
└─────────────────────────────┼──────────────────────────┘
                              │ Seam: Box<dyn AudioSink>
                              ▼
┌────────────────────────────────────────────────────────┐
│                  AudioSink Adapters                    │
│                                                        │
│  ┌───────────────────────┐  ┌───────────────────────┐  │
│  │       RodioSink       │  │        MockSink       │  │
│  │   (Production Device) │  │    (Automated Tests)  │  │
│  │  - rodio::OutputStream│  │  - No audio drivers   │  │
│  │  - rodio::Sink        │  │  - Records commands   │  │
│  │  - cpal audio output  │  │  - Zero thread sleeps │  │
│  └───────────────────────┘  └───────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

## Seam & Interface Definition

### `AudioSink` Trait

```rust
pub trait AudioSink: Send {
    fn play(&mut self);
    fn pause(&mut self);
    fn stop(&mut self);
    fn set_volume(&mut self, volume: f32);
    fn is_empty(&self) -> bool;
    fn append_source(&mut self, source: Box<dyn rodio::Source<Item = f32> + Send>);
}
```

### Adapters

#### 1. `RodioSink` (Production Adapter)
Wraps `rodio::OutputStream` and dynamically managed `rodio::Sink`. Re-instantiates sinks cleanly upon seek or source load, properly routing OS device errors.

#### 2. `MockSink` (Test Adapter)
Maintains internal telemetry:
```rust
#[derive(Default)]
pub struct MockSink {
    pub is_playing: bool,
    pub volume: f32,
    pub append_count: usize,
    pub empty: bool,
}
```
Executes synchronously in memory, enabling instant assertions without hardware initialization or timeouts.

## Key Technical Decisions

### 1. Reclaiming `AudioEngine`
Instead of having `AudioWorker` maintain its own private `PlaybackState`, `AudioEngine` is revived to manage:
- Handling all `PlayerCommand`s.
- Updating `PlaybackState` deterministically.
- Directing sink actions (`sink.play()`, `sink.pause()`, `sink.set_volume(...)`).
`AudioWorker` becomes a lightweight thread runner whose sole responsibility is driving `AudioEngine.tick()` and polling the command channel.

### 2. Eliminating Test Sleep Delays
Tests in `crates/audio/tests/controller_test.rs` currently contain:
```rust
std::thread::sleep(Duration::from_millis(300));
```
With `MockSink`, tests can instantiate `AudioEngine::new_with_sink(Box::new(MockSink::new()))` and verify:
- Volume and mute transitions in 0ms.
- Seek state progression in 0ms.
- Decoder attachment without hardware audio sinks.

## Error Handling & Hardware Fallbacks

If `RodioSink::try_new()` fails (e.g. running in an audio-less Linux container), `AudioWorker` falls back cleanly to a `NullSink` adapter that records time progression without panicking, ensuring Spoty can run headlessly in all environments.
