## 1. Spotify Client & Provider Implementation (`spoty-storage`)

- [x] 1.1 Implement `SpotifyConfig` with `client_id`, `client_secret`, and `device_name`.
- [x] 1.2 Implement `SpotifyProvider` with token exchange (`accounts.spotify.com/api/token`), search, and library sync.
- [x] 1.3 Implement stream acquisition for Spotify tracks via preview URLs or stream proxy.
- [x] 1.4 Add unit tests verifying `SpotifyConfig` serialization and token exchange payload formatting. Verify with `cargo test -p spoty-storage`.

## 2. Settings UI & Provider Integration (`spoty-ui`)

- [x] 2.1 Update `SpotyConfig` to include `SpotifyConfig`.
- [x] 2.2 Add Spotify Connect configuration form in `SettingsView` with Client ID, Client Secret, and Connect button.
- [x] 2.3 Wire Spotify provider activation in `SpotyApp` to reload catalog and update active source badge.
- [x] 2.4 Run full workspace test suite and build verification with `cargo test --workspace`.
