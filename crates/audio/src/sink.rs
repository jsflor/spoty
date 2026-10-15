use std::sync::Arc;
use std::time::Duration;

/// Trait representing an audio hardware output sink.
pub trait AudioSink: Send {
    fn play(&mut self);
    fn pause(&mut self);
    fn stop(&mut self);
    fn set_volume(&mut self, volume: f32);
    fn volume(&self) -> f32;
    fn is_empty(&self) -> bool;
    fn append_source(&mut self, source: Box<dyn rodio::Source<Item = f32> + Send>);
    fn create_secondary(&self) -> Box<dyn AudioSink>;
}

/// Thin wrapper that allows a boxed `dyn rodio::Source<Item = f32> + Send`
/// to be passed directly to `rodio::Sink::append`.
pub struct BoxedSource(pub Box<dyn rodio::Source<Item = f32> + Send>);

impl Iterator for BoxedSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl rodio::Source for BoxedSource {
    fn current_frame_len(&self) -> Option<usize> {
        self.0.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.0.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.0.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.0.total_duration()
    }
}

struct RodioStreamContext {
    _stream: rodio::OutputStream,
}

// SAFETY: `OutputStream` contains a cpal `Stream` which has `PhantomData<*mut ()>` to denote
// platform non-portability. Sharing `RodioStreamContext` across worker threads and sinks is safe.
unsafe impl Send for RodioStreamContext {}
unsafe impl Sync for RodioStreamContext {}

/// Production audio sink adapter backed by `rodio::OutputStream` and `rodio::Sink`.
pub struct RodioSink {
    stream_ctx: Arc<RodioStreamContext>,
    stream_handle: rodio::OutputStreamHandle,
    sink: Option<rodio::Sink>,
    volume: f32,
}

// SAFETY: `OutputStream` contains a cpal `Stream` which has `PhantomData<*mut ()>` to denote
// platform non-portability. Moving `RodioSink` into the audio worker thread is safe.
unsafe impl Send for RodioSink {}

impl RodioSink {
    pub fn try_new() -> anyhow::Result<Self> {
        let (stream, stream_handle) = rodio::OutputStream::try_default()?;
        let sink = rodio::Sink::try_new(&stream_handle).ok();
        let stream_ctx = Arc::new(RodioStreamContext { _stream: stream });
        Ok(Self {
            stream_ctx,
            stream_handle,
            sink,
            volume: 1.0,
        })
    }

    pub fn recreate_sink(&mut self) {
        self.sink = rodio::Sink::try_new(&self.stream_handle).ok();
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }
}

impl AudioSink for RodioSink {
    fn play(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }

    fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.recreate_sink();
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(sink) = &self.sink {
            sink.set_volume(volume);
        }
    }

    fn volume(&self) -> f32 {
        self.volume
    }

    fn is_empty(&self) -> bool {
        self.sink.as_ref().map_or(true, |s| s.empty())
    }

    fn append_source(&mut self, source: Box<dyn rodio::Source<Item = f32> + Send>) {
        if self.sink.is_none() {
            self.recreate_sink();
        }
        if let Some(sink) = &self.sink {
            sink.append(BoxedSource(source));
        }
    }

    fn create_secondary(&self) -> Box<dyn AudioSink> {
        let sink = rodio::Sink::try_new(&self.stream_handle).ok();
        Box::new(RodioSink {
            stream_ctx: Arc::clone(&self.stream_ctx),
            stream_handle: self.stream_handle.clone(),
            sink,
            volume: self.volume,
        })
    }
}

/// In-memory mock audio sink for deterministic, zero-sleep unit and integration tests.
#[derive(Debug, Clone)]
pub struct MockSink {
    pub is_playing: bool,
    pub volume: f32,
    pub append_count: usize,
    pub empty: bool,
}

impl Default for MockSink {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSink {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            volume: 1.0,
            append_count: 0,
            empty: true,
        }
    }

    pub fn set_empty(&mut self, empty: bool) {
        self.empty = empty;
    }
}

impl AudioSink for MockSink {
    fn play(&mut self) {
        self.is_playing = true;
    }

    fn pause(&mut self) {
        self.is_playing = false;
    }

    fn stop(&mut self) {
        self.is_playing = false;
        self.empty = true;
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn volume(&self) -> f32 {
        self.volume
    }

    fn is_empty(&self) -> bool {
        self.empty
    }

    fn append_source(&mut self, _source: Box<dyn rodio::Source<Item = f32> + Send>) {
        self.append_count += 1;
        self.empty = false;
    }

    fn create_secondary(&self) -> Box<dyn AudioSink> {
        Box::new(MockSink::new())
    }
}

/// Silent no-op audio sink fallback for headless or driverless environments.
#[derive(Debug, Default, Clone)]
pub struct NullSink {
    volume: f32,
}

impl NullSink {
    pub fn new() -> Self {
        Self { volume: 1.0 }
    }
}

impl AudioSink for NullSink {
    fn play(&mut self) {}

    fn pause(&mut self) {}

    fn stop(&mut self) {}

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn volume(&self) -> f32 {
        self.volume
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn append_source(&mut self, _source: Box<dyn rodio::Source<Item = f32> + Send>) {}

    fn create_secondary(&self) -> Box<dyn AudioSink> {
        Box::new(NullSink::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_sink_operations() {
        let mut sink = MockSink::new();
        assert!(sink.is_empty());
        assert!(!sink.is_playing);

        sink.play();
        assert!(sink.is_playing);

        sink.pause();
        assert!(!sink.is_playing);

        sink.set_volume(0.42);
        assert!((sink.volume - 0.42).abs() < 1e-5);

        sink.stop();
        assert!(sink.is_empty());
        assert!(!sink.is_playing);
    }

    #[test]
    fn test_null_sink_operations() {
        let mut sink = NullSink::new();
        assert!(sink.is_empty());
        sink.play();
        sink.set_volume(0.5);
        sink.pause();
        sink.stop();
        assert!(sink.is_empty());
    }
}
