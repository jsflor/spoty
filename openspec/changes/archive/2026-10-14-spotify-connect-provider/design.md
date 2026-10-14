## Context

See `proposal.md` for motivation. We are adding an official Spotify integration provider so users can combine their self-hosted tracks with official Spotify music.

## Goals / Non-Goals

**Goals:**
- Provide `SpotifyConfig` with `client_id`, `client_secret`, and `device_name`.
- Authenticate against Spotify's OAuth token endpoint (`https://accounts.spotify.com/api/token`).
- Implement `SpotifyProvider` adhering strictly to `MusicProvider`.
- Expose search and playlist queries directly against `https://api.spotify.com/v1/`.
- Provide preview audio streaming for tracks (`preview_url`) or Connect session fallback.

**Non-Goals:**
- Circumventing Widevine DRM or downloading DRM-protected master audio (strictly out of scope and against policy).
- Reverse-engineering Spotify internal binaries.

## Decisions

### 1. Official Spotify Web API with Token Refresh
- **Decision:** Use Spotify's official Web API over HTTPS via `ureq` with automatic token refresh.
- **Rationale:** Standard, fully supported by Spotify Developer Terms, zero risk of account ban or DRM issues.

### 2. Audio Playback Strategy
- **Decision:** Stream 30s official audio previews or Connect audio stream when enabled, seamlessly flowing into Spoty's `rodio` audio pipeline.
- **Rationale:** Ensures legal compliance and universal compatibility across all platforms.

## Risks / Trade-offs

- **[Risk]** Spotify rate limits (429 Too Many Requests).
  → **Mitigation**: Cache library responses in `library_cache` with exponential backoff on retry.
