# storage/metadata-tag-extraction Specification

## ADDED Requirements

### Requirement: Directory Path Metadata Heuristics
The metadata extraction engine SHALL inspect the file's parent directory hierarchy to infer artist and album names when embedded ID3 or Vorbis tags are absent.

#### Scenario: Extracting tags from hierarchical folder structure
- **GIVEN** an audio file with path `Music/Daft Punk/Discovery/01 One More Time.mp3` with no embedded tags
- **WHEN** `extract_audio_metadata` is called with the file path
- **THEN** it infers Artist as `Daft Punk`, Album as `Discovery`, and Title as `One More Time`
