use spoty_audio::{
    AudioStreamReader, AudioStreamResolver, BoundedStreamReader, PlaybackSession, SessionEvent,
};
use spoty_core::{AudioFormat, PlaybackStatus, Track};
use spoty_storage::{DemoProvider, MusicProvider};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;

struct DemoResolver {
    provider: Arc<DemoProvider>,
    handle: tokio::runtime::Handle,
}

impl AudioStreamResolver for DemoResolver {
    fn resolve_stream(&self, track: &Track) -> anyhow::Result<Box<dyn AudioStreamReader>> {
        let stream = self
            .handle
            .block_on(self.provider.stream_track(&track.id))?;
        let (tx, rx) = std::sync::mpsc::sync_channel(4);
        let mut async_stream = stream;
        self.handle.spawn(async move {
            let mut buf = vec![0u8; 64 * 1024];
            while let Ok(n) = async_stream.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                if tx.send(Ok(buf[..n].to_vec())).is_err() {
                    break;
                }
            }
        });
        Ok(Box::new(BoundedStreamReader::new(rx)))
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_demo_tracks_end_to_end_playback() {
    let provider = Arc::new(DemoProvider::new());
    let library = provider.fetch_library().await.unwrap();

    // Verify 3 CC-BY tracks with Kevin MacLeod attribution
    assert_eq!(library.tracks.len(), 3);
    for track in &library.tracks {
        assert_eq!(track.artist, "Kevin MacLeod");
        assert_eq!(track.format, AudioFormat::Ogg);
    }

    let resolver = Arc::new(DemoResolver {
        provider: Arc::clone(&provider),
        handle: tokio::runtime::Handle::current(),
    });

    let (session, event_rx) = PlaybackSession::new(resolver);

    // Play all demo tracks starting at index 0
    session.play_all(library.tracks.clone(), 0);

    let mut track_playing = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(500))
        {
            if state.playback.status == PlaybackStatus::Playing
                && state.current_track.as_ref().map(|t| t.title.as_str())
                    == Some("Consort for Brass")
            {
                track_playing = true;
                break;
            }
        }
    }
    assert!(
        track_playing,
        "Expected first demo track to start playing via PlaybackSession"
    );

    // Test forward seek on compressed Ogg Vorbis track
    session.seek(Duration::from_secs(20));

    let mut forward_seek_ok = false;
    for _ in 0..10 {
        if let Ok(event) = event_rx.recv_timeout(Duration::from_millis(500)) {
            match event {
                SessionEvent::StateChanged(st) => {
                    assert_eq!(st.playback.status, PlaybackStatus::Playing);
                    if (st.playback.position_secs - 20.0).abs() < 1.0 {
                        forward_seek_ok = true;
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    assert!(
        forward_seek_ok,
        "Forward seek on Ogg track failed to update position"
    );

    // Test backward seek on compressed Ogg Vorbis track
    session.seek(Duration::from_secs(5));

    let mut backward_seek_ok = false;
    for _ in 0..10 {
        if let Ok(event) = event_rx.recv_timeout(Duration::from_millis(500)) {
            match event {
                SessionEvent::StateChanged(st) => {
                    assert_eq!(st.playback.status, PlaybackStatus::Playing);
                    if (st.playback.position_secs - 5.0).abs() < 1.0 {
                        backward_seek_ok = true;
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    assert!(
        backward_seek_ok,
        "Backward seek on Ogg track failed to update position"
    );

    session.pause();
}
