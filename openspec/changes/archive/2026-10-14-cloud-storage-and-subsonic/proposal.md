## Why

To fulfill the user's vision of an open-source, self-hosted streaming client where users decide where their music is stored (Cloudflare R2, AWS S3, home servers running Navidrome/Jellyfin on a Raspberry Pi), we need first-class S3-compatible cloud storage support and the Subsonic API protocol.

## What Changes

- Enable OpenDAL's `services-s3` feature in `spoty-storage` and implement `S3Config` for AWS S3 and Cloudflare R2.
- Implement `SubsonicProvider` in `spoty-storage` to connect to Navidrome, Jellyfin, and Subsonic-compatible home servers.
- Add configuration models and persistent storage settings in `spoty-core` and `spoty-ui`.
- Expand `SettingsView` with tabs/forms for Local Filesystem, S3 / Cloudflare R2, and Navidrome / Subsonic servers with live test-connection validation.

## Capabilities

### New Capabilities
- `storage/s3-cloud-storage`: OpenDAL S3 service integration supporting bucket scanning, credentials authentication, custom endpoints (Cloudflare R2), and audio byte streaming.
- `storage/subsonic-provider`: Subsonic REST client supporting authentication (token + salt), playlist synchronization, search, and remote stream resolution.

### Modified Capabilities
<!-- None -->

## Impact

- Enables `services-s3` in `opendal`.
- Extends `SettingsView` with cloud and server connection forms.
- Allows streaming directly from remote servers and cloud buckets without downloading files in advance.
