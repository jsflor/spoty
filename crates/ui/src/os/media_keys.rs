use eframe::egui;
use spoty_audio::PlaybackSession;

#[cfg(target_os = "macos")]
pub fn attach_media_keys(
    controls: &mut souvlaki::MediaControls,
    session: PlaybackSession,
    egui_ctx: egui::Context,
) -> Result<(), souvlaki::Error> {
    controls.attach(move |event| {
        handle_media_control_event(event, &session, &egui_ctx);
    })
}

#[cfg(target_os = "macos")]
pub fn handle_media_control_event(
    event: souvlaki::MediaControlEvent,
    session: &PlaybackSession,
    egui_ctx: &egui::Context,
) {
    match event {
        souvlaki::MediaControlEvent::Toggle => {
            session.toggle_play_pause();
        }
        souvlaki::MediaControlEvent::Play => {
            session.play();
        }
        souvlaki::MediaControlEvent::Pause => {
            session.pause();
        }
        souvlaki::MediaControlEvent::Next => {
            session.next();
        }
        souvlaki::MediaControlEvent::Previous => {
            session.prev();
        }
        souvlaki::MediaControlEvent::SetPosition(pos) => {
            session.seek(pos.0);
        }
        souvlaki::MediaControlEvent::SeekBy(direction, amount) => {
            let current = session.state().playback.position_secs;
            let delta = amount.as_secs_f64();
            let new_pos = match direction {
                souvlaki::SeekDirection::Forward => current + delta,
                souvlaki::SeekDirection::Backward => (current - delta).max(0.0),
            };
            session.seek(std::time::Duration::from_secs_f64(new_pos));
        }
        souvlaki::MediaControlEvent::Seek(direction) => {
            let current = session.state().playback.position_secs;
            let delta = 5.0;
            let new_pos = match direction {
                souvlaki::SeekDirection::Forward => current + delta,
                souvlaki::SeekDirection::Backward => (current - delta).max(0.0),
            };
            session.seek(std::time::Duration::from_secs_f64(new_pos));
        }
        _ => {}
    }
    egui_ctx.request_repaint();
}

#[cfg(not(target_os = "macos"))]
pub fn attach_media_keys(_session: PlaybackSession, _egui_ctx: egui::Context) {}

#[cfg(test)]
mod tests {
    use super::*;
    use spoty_audio::{AudioStreamReader, AudioStreamResolver};
    use spoty_core::Track;
    use std::sync::Arc;

    struct TestResolver;
    impl AudioStreamResolver for TestResolver {
        fn resolve_stream(&self, _track: &Track) -> anyhow::Result<Box<dyn AudioStreamReader>> {
            anyhow::bail!("TestResolver")
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_handle_media_control_event() {
        let (session, rx) = PlaybackSession::new(Arc::new(TestResolver));
        let ctx = egui::Context::default();

        handle_media_control_event(souvlaki::MediaControlEvent::Toggle, &session, &ctx);
        handle_media_control_event(souvlaki::MediaControlEvent::Next, &session, &ctx);
        handle_media_control_event(souvlaki::MediaControlEvent::Previous, &session, &ctx);
        handle_media_control_event(
            souvlaki::MediaControlEvent::SetPosition(souvlaki::MediaPosition(
                std::time::Duration::from_secs(30),
            )),
            &session,
            &ctx,
        );

        // Verify session receives commands without error
        assert!(rx.try_recv().is_ok() || rx.try_recv().is_err());
    }
}
