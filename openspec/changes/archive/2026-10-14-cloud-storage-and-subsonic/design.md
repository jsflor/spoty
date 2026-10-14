## Context

In Milestones 1–3, we established the core domain, demo provider, local filesystem scanning, background audio thread, and the full multi-view UI. Milestone 4 expands storage into cloud buckets and self-hosted server protocols.

## Goals / Non-Goals

**Goals:**
- Add `S3Config` (`endpoint`, `bucket`, `region`, `access_key_id`, `secret_access_key`) and support Cloudflare R2 / AWS S3 in `OpenDalProvider`.
- Implement `SubsonicConfig` and `SubsonicProvider` for connecting to self-hosted Navidrome / Jellyfin servers on home hardware (e.g. Raspberry Pi).
- Build interactive configuration forms in `SettingsView` with live validation and catalog reloading.
- Provide persistent configuration loading/saving (`spoty_config.json`).

**Non-Goals:**
- Bidirectional playlist writing to remote Subsonic servers (read-only stream and catalog sync).
- Direct peer-to-peer WebRTC streaming.

## Decisions

### 1. Unified OpenDAL S3 Operator
- **Decision:** Use OpenDAL's `services::S3` backend directly inside `OpenDalProvider`.
- **Rationale:** OpenDAL abstracts S3 and Cloudflare R2 into the same `Operator` interface we already use for local filesystem, sharing all recursive directory scanning and stream resolution code.

### 2. Subsonic Protocol Integration
- **Decision:** Use lightweight HTTP calls (`ureq` or `reqwest` or `tokio`) with standard Subsonic JSON endpoints (`/rest/ping.view`, `/rest/getPlaylists.view`, `/rest/search3.view`, `/rest/stream.view`).
- **Rationale:** Subsonic is supported by virtually all open-source self-hosted music servers (Navidrome, Jellyfin, Airsonic, Funkwhale).

## Risks / Trade-offs

- **[Risk]** Heavy dependencies for S3 (AWS SDKs).
  → **Mitigation**: OpenDAL's `services-s3` feature is built on top of standard HTTP clients without pulling the entire monolithic AWS SDK.
