use spoty_audio::SessionState;
use spoty_core::PlaybackStatus;
use std::time::Duration;

#[cfg(target_os = "macos")]
pub fn update_now_playing(
    controls: &mut souvlaki::MediaControls,
    state: &SessionState,
) -> Result<(), souvlaki::Error> {
    if let Some(track) = &state.current_track {
        let duration_secs = if state.playback.duration_secs > 0.0 {
            state.playback.duration_secs
        } else {
            track.duration_secs as f64
        };

        controls.set_metadata(souvlaki::MediaMetadata {
            title: Some(&track.title),
            artist: Some(&track.artist),
            album: track.album.as_deref(),
            duration: Some(Duration::from_secs_f64(duration_secs)),
            cover_url: track.cover_uri.as_deref(),
        })?;
    } else {
        controls.set_metadata(souvlaki::MediaMetadata::default())?;
    }

    let pos = Duration::from_secs_f64(state.playback.position_secs.max(0.0));
    let playback = match state.playback.status {
        PlaybackStatus::Playing => souvlaki::MediaPlayback::Playing {
            progress: Some(souvlaki::MediaPosition(pos)),
        },
        PlaybackStatus::Paused => souvlaki::MediaPlayback::Paused {
            progress: Some(souvlaki::MediaPosition(pos)),
        },
        PlaybackStatus::Stopped | PlaybackStatus::Buffering => souvlaki::MediaPlayback::Stopped,
    };
    controls.set_playback(playback)?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn update_now_playing(_state: &SessionState) {}

#[cfg(test)]
mod tests {
    use super::*;
    use spoty_core::{AudioFormat, Track, TrackId};

    #[test]
    fn test_now_playing_helpers() {
        let mut state = SessionState::default();
        assert_eq!(state.playback.status, PlaybackStatus::Stopped);
        assert!(state.current_track.is_none());

        let track = Track {
            id: TrackId::new("t1"),
            title: "Test Track".into(),
            artist: "Test Artist".into(),
            album: Some("Test Album".into()),
            duration_secs: 180,
            cover_uri: None,
            source_uri: "demo://test.wav".into(),
            format: AudioFormat::Wav,
        };
        state.current_track = Some(track);
        state.playback.status = PlaybackStatus::Playing;
        state.playback.position_secs = 45.0;
        state.playback.duration_secs = 180.0;

        assert_eq!(state.current_track.as_ref().unwrap().title, "Test Track");
        assert_eq!(state.playback.position_secs, 45.0);
    }
}
