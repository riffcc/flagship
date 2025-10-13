use lens_node::audio_metadata::{is_audio_file, is_image_file};

#[test]
fn test_audio_file_detection() {
    // Valid audio files
    assert!(is_audio_file("song.mp3"));
    assert!(is_audio_file("track.flac"));
    assert!(is_audio_file("audio.m4a"));
    assert!(is_audio_file("file.aac"));
    assert!(is_audio_file("music.ogg"));
    assert!(is_audio_file("track.opus"));
    assert!(is_audio_file("audio.wav"));
    assert!(is_audio_file("SONG.MP3")); // Case insensitive

    // Invalid audio files
    assert!(!is_audio_file("video.mp4"));
    assert!(!is_audio_file("document.pdf"));
    assert!(!is_audio_file("image.jpg"));
}

#[test]
fn test_image_file_detection() {
    // Valid image files
    assert!(is_image_file("cover.jpg"));
    assert!(is_image_file("artwork.jpeg"));
    assert!(is_image_file("image.png"));
    assert!(is_image_file("photo.webp"));
    assert!(is_image_file("COVER.JPG")); // Case insensitive

    // Invalid image files
    assert!(!is_image_file("song.mp3"));
    assert!(!is_image_file("video.mp4"));
    assert!(!is_image_file("document.pdf"));
}

#[test]
fn test_supported_formats() {
    // Test all supported audio formats
    let audio_formats = vec![
        "mp3", "flac", "m4a", "aac", "ogg", "opus", "wav", "wma", "ape", "wv",
    ];

    for format in audio_formats {
        let filename = format!("test.{}", format);
        assert!(is_audio_file(&filename), "Format {} should be supported", format);
    }

    // Test all supported image formats
    let image_formats = vec!["jpg", "jpeg", "png", "webp", "gif"];

    for format in image_formats {
        let filename = format!("test.{}", format);
        assert!(is_image_file(&filename), "Format {} should be supported", format);
    }
}
