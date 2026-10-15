## MODIFIED Requirements

### Requirement: Demo Audio Asset Resolution
The system SHALL provide a default bundled demo provider that serves offline authentic Creative Commons royalty-free audio tracks with embedded metadata and album art without requiring cloud configuration or internet access.

#### Scenario: Playing bundled demo track
- **WHEN** the demo provider is requested to stream a demo track by ID
- **THEN** it resolves a readable byte stream of valid Ogg Vorbis audio data ready for decoding

#### Scenario: Cataloging demo tracks with metadata
- **WHEN** the demo provider library is fetched
- **THEN** it returns tracks with valid titles, artist attribution, album names, durations, and cover image URIs
