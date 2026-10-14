## Purpose

Provides integration with Subsonic API servers, enabling connection to Navidrome, Jellyfin, and Airsonic instances running on home servers or Raspberry Pis.

## ADDED Requirements

### Requirement: Subsonic Server Authentication
The system SHALL authenticate with Subsonic API endpoints using token and salt authentication parameters (`u`, `t`, `s`, `v`, `c`).

#### Scenario: Server ping handshake
- **WHEN** user tests connection to a Subsonic or Navidrome URL with valid credentials
- **THEN** the system executes a ping handshake and verifies the server response

### Requirement: Remote Catalog Discovery and Stream Proxy
The system SHALL retrieve playlists and search results from the Subsonic server and stream media via the `stream.view` endpoint.

#### Scenario: Playing a track from Navidrome
- **WHEN** user initiates playback on a Subsonic track
- **THEN** the provider resolves an `AudioStream` fetching audio bytes from the server's streaming endpoint
