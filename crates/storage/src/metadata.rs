use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::io::Cursor;

#[derive(Debug, Clone, Default)]
pub struct ExtractedMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_secs: u32,
    pub cover_art: Option<Vec<u8>>,
}

/// Parses embedded audio tags and duration from in-memory audio bytes, with fallback heuristics
pub fn extract_audio_metadata(bytes: &[u8], fallback_filename: &str) -> ExtractedMetadata {
    // Attempt parsing with lofty
    let parsed = (|| -> Option<ExtractedMetadata> {
        let mut cursor = Cursor::new(bytes);
        let tagged_file = Probe::new(&mut cursor)
            .guess_file_type()
            .ok()?
            .read()
            .ok()?;

        let properties = tagged_file.properties();
        let duration_secs = properties.duration().as_secs() as u32;

        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag());

        if let Some(tag) = tag {
            let title = tag.title().map(|s| s.to_string()).filter(|s| !s.is_empty());
            let artist = tag
                .artist()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
            let album = tag.album().map(|s| s.to_string()).filter(|s| !s.is_empty());
            let cover_art = tag.pictures().first().map(|p| p.data().to_vec());

            let (fb_artist, fb_title) = parse_filename_heuristics(fallback_filename);

            Some(ExtractedMetadata {
                title: title.unwrap_or(fb_title),
                artist: artist.unwrap_or(fb_artist),
                album,
                duration_secs: if duration_secs > 0 {
                    duration_secs
                } else {
                    180
                },
                cover_art,
            })
        } else {
            let (fb_artist, fb_title) = parse_filename_heuristics(fallback_filename);
            Some(ExtractedMetadata {
                title: fb_title,
                artist: fb_artist,
                album: None,
                duration_secs: if duration_secs > 0 {
                    duration_secs
                } else {
                    180
                },
                cover_art: None,
            })
        }
    })();

    parsed.unwrap_or_else(|| {
        let (artist, title) = parse_filename_heuristics(fallback_filename);
        ExtractedMetadata {
            title,
            artist,
            album: None,
            duration_secs: 180,
            cover_art: None,
        }
    })
}

pub fn parse_filename_heuristics(stem: &str) -> (String, String) {
    if let Some((artist, title)) = stem.split_once(" - ") {
        (artist.trim().to_string(), title.trim().to_string())
    } else if let Some((artist, title)) = stem.split_once('_') {
        (artist.trim().to_string(), title.trim().to_string())
    } else {
        ("Unknown Artist".to_string(), stem.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_audio_metadata_fallback() {
        let dummy_bytes = b"not_a_real_audio_stream";
        let meta = extract_audio_metadata(dummy_bytes, "Daft Punk - Harder Better Faster");
        assert_eq!(meta.artist, "Daft Punk");
        assert_eq!(meta.title, "Harder Better Faster");
        assert_eq!(meta.duration_secs, 180);
        assert!(meta.cover_art.is_none());
    }
}
