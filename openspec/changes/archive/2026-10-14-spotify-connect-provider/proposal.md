## Why

Users who already have Spotify accounts want the option to use Spotify as a music source alongside their self-hosted storage backends (Local, S3/R2, Navidrome). Integrating official Spotify authentication, metadata sync, and Connect streaming allows users to aggregate all their music into Spoty without vendor lock-in.

## What Changes

- Implement `SpotifyConfig` in `spoty-storage` and `spoty-ui` supporting Client Credentials, user access tokens, and Connect device settings.
- Implement `SpotifyProvider` implementing `MusicProvider` with official Spotify Web API catalog synchronization, global search, and audio stream resolution.
- Add Spotify configuration form to `SettingsView` with account connection status and token persistence.

## Capabilities

### New Capabilities
- `storage/spotify-provider`: Official Spotify API integration supporting OAuth authentication, playlist and liked tracks synchronization, Spotify catalog search, and audio stream resolution.

### Modified Capabilities
<!-- None -->

## Impact

- Extends `spoty-storage` with `SpotifyProvider`.
- Adds Spotify account settings to `SpotyConfig` and `SettingsView`.
- Allows users to search and play tracks from Spotify alongside self-hosted music.
