use spoty_storage::{MusicProvider, OpenDalProvider};
use std::fs::File;
use std::io::Write;
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn test_opendal_provider_fs_scan_and_stream() {
    let temp_dir = std::env::temp_dir().join(format!("spoty_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    // Create a mock audio file in the temp dir: "Artist - Song.wav"
    let file_path = temp_dir.join("Daft Punk - One More Time.wav");
    let mut file = File::create(&file_path).unwrap();
    // Write a dummy WAV header
    file.write_all(b"RIFF\x24\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x02\x00\x44\xac\x00\x00\x10\xb1\x02\x00\x04\x00\x10\x00data\x00\x00\x00\x00").unwrap();

    let provider = OpenDalProvider::new_fs(&temp_dir).unwrap();
    let library = provider.fetch_library().await.unwrap();

    assert_eq!(library.tracks.len(), 1);
    let track = &library.tracks[0];
    assert_eq!(track.artist, "Daft Punk");
    assert_eq!(track.title, "One More Time");

    // Test stream resolution
    let mut stream = provider.stream_track(&track.id).await.unwrap();
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await.unwrap();
    assert_eq!(&header, b"RIFF");

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}
