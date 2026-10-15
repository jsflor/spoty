pub mod media_keys;
pub mod now_playing;

use eframe::egui;
use spoty_audio::{PlaybackSession, SessionState};

pub struct OsIntegration {
    #[cfg(target_os = "macos")]
    controls: Option<souvlaki::MediaControls>,
}

impl OsIntegration {
    pub fn new(session: PlaybackSession, egui_ctx: egui::Context) -> Self {
        #[cfg(target_os = "macos")]
        {
            let config = souvlaki::PlatformConfig {
                dbus_name: "spoty",
                display_name: "Spoty",
                hwnd: None,
            };
            match souvlaki::MediaControls::new(config) {
                Ok(mut controls) => {
                    if let Err(e) = media_keys::attach_media_keys(&mut controls, session, egui_ctx)
                    {
                        tracing::warn!("Failed to attach macOS media keys: {e:?}");
                    }
                    Self {
                        controls: Some(controls),
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize macOS media controls: {e:?}");
                    Self { controls: None }
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (session, egui_ctx);
            Self {}
        }
    }

    pub fn update(&mut self, state: &SessionState) {
        #[cfg(target_os = "macos")]
        {
            if let Some(controls) = &mut self.controls {
                if let Err(e) = now_playing::update_now_playing(controls, state) {
                    tracing::warn!("Failed to update macOS now playing info: {e:?}");
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = state;
        }
    }
}

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
    fn test_os_integration_lifecycle() {
        let (session, _) = PlaybackSession::new(Arc::new(TestResolver));
        let ctx = egui::Context::default();
        let mut os_integration = OsIntegration::new(session, ctx);

        let state = SessionState::default();
        os_integration.update(&state);
    }
}
