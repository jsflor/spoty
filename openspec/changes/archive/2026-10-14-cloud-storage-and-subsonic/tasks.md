## 1. S3 & Cloudflare R2 Storage Support (`spoty-storage`)

- [x] 1.1 Enable `services-s3` in OpenDAL inside `Cargo.toml`.
- [x] 1.2 Implement `S3Config` struct and `OpenDalProvider::new_s3(...)` constructor supporting AWS S3 and Cloudflare R2 endpoints.
- [x] 1.3 Add unit tests verifying `S3Config` parsing and builder construction. Verify with `cargo test -p spoty-storage`.

## 2. Subsonic / Navidrome Provider (`spoty-storage`)

- [x] 2.1 Implement `SubsonicConfig` with URL, username, password, and token/salt MD5 authentication hashing.
- [x] 2.2 Implement `SubsonicProvider` implementing `MusicProvider` with `fetch_library`, `search`, and `stream_track`.
- [x] 2.3 Add unit tests verifying Subsonic authentication hashing and endpoint URL generation. Verify with `cargo test -p spoty-storage`.

## 3. Persistent Settings & UI Form Controls (`spoty-ui`)

- [x] 3.1 Implement `SpotyConfig` model with serde serialization for saving/loading provider preferences.
- [x] 3.2 Update `SettingsView` with tabs/forms for Local Directory, S3/Cloudflare R2, and Subsonic/Navidrome.
- [x] 3.3 Connect settings provider activation to live UI library reloading and playback queue refresh.
- [x] 3.4 Run full workspace test suite and build verification with `cargo test --workspace`.
