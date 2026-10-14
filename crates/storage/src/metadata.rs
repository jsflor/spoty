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
pub fn extract_audio_metadata(bytes: &[u8], fallback_path: &str) -> ExtractedMetadata {
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

        let (fb_artist, fb_title, fb_album) = parse_path_heuristics(fallback_path);

        if let Some(tag) = tag {
            let title = tag.title().map(|s| s.to_string()).filter(|s| !s.is_empty());
            let artist = tag
                .artist()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
            let album = tag.album().map(|s| s.to_string()).filter(|s| !s.is_empty());
            let cover_art = tag.pictures().first().map(|p| p.data().to_vec());

            Some(ExtractedMetadata {
                title: title.unwrap_or(fb_title),
                artist: artist.unwrap_or(fb_artist),
                album: album.or(fb_album),
                duration_secs: if duration_secs > 0 {
                    duration_secs
                } else {
                    180
                },
                cover_art,
            })
        } else {
            Some(ExtractedMetadata {
                title: fb_title,
                artist: fb_artist,
                album: fb_album,
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
        let (artist, title, album) = parse_path_heuristics(fallback_path);
        ExtractedMetadata {
            title,
            artist,
            album,
            duration_secs: 180,
            cover_art: None,
        }
    })
}

/// Infers Artist, Title, and optional Album from directory hierarchy or filename
pub fn parse_path_heuristics(path_str: &str) -> (String, String, Option<String>) {
    let p = std::path::Path::new(path_str);
    let raw_stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(path_str);
    let clean_title = strip_track_number_prefix(raw_stem);

    let components: Vec<&str> = p
        .parent()
        .map(|parent| parent.iter().filter_map(|c| c.to_str()).collect())
        .unwrap_or_default();

    if components.len() >= 2 {
        let artist = components[components.len() - 2].trim().to_string();
        let album = components[components.len() - 1].trim().to_string();
        (artist, clean_title, Some(album))
    } else if components.len() == 1 && !is_generic_folder(components[0]) {
        let artist = components[0].trim().to_string();
        (artist, clean_title, None)
    } else {
        let (artist, title) = parse_filename_heuristics(raw_stem);
        (artist, title, None)
    }
}

fn strip_track_number_prefix(stem: &str) -> String {
    let bytes = stem.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i > 0 && i < bytes.len() {
        let rest = &stem[i..];
        if let Some(stripped) = rest.strip_prefix(" - ") {
            return stripped.trim().to_string();
        } else if let Some(stripped) = rest.strip_prefix(". ") {
            return stripped.trim().to_string();
        } else if let Some(stripped) = rest.strip_prefix('.') {
            return stripped.trim().to_string();
        } else if let Some(stripped) = rest.strip_prefix(' ') {
            return stripped.trim().to_string();
        } else if let Some(stripped) = rest.strip_prefix('_') {
            return stripped.trim().to_string();
        }
    }
    stem.to_string()
}

fn is_generic_folder(name: &str) -> bool {
    let n = name.to_lowercase();
    matches!(
        n.as_str(),
        "music" | "downloads" | "songs" | "audio" | "tracks" | "." | ""
    )
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

    #[test]
    fn test_parse_path_heuristics_hierarchical() {
        let (artist, title, album) =
            parse_path_heuristics("Music/Pink Floyd/The Wall/01 In The Flesh.flac");
        assert_eq!(artist, "Pink Floyd");
        assert_eq!(album, Some("The Wall".to_string()));
        assert_eq!(title, "In The Flesh");

        let (artist2, title2, album2) =
            parse_path_heuristics("Radiohead/02 - Paranoid Android.mp3");
        assert_eq!(artist2, "Radiohead");
        assert_eq!(album2, None);
        assert_eq!(title2, "Paranoid Android");
    }
}
