use anyhow::{Context, Result};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::ItemKey;
use std::path::Path;

/// Audio metadata extracted from audio files
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioMetadata {
    /// Duration in seconds
    pub duration_secs: f64,
    /// Sample rate in Hz
    pub sample_rate: Option<u32>,
    /// Number of channels
    pub channels: Option<u8>,
    /// Bitrate in bits per second
    pub bitrate: Option<u32>,
    /// File format/codec
    pub format: String,
    /// Blake3 hash of file content
    pub blake3_hash: String,
    /// ID3/metadata tags
    pub tags: AudioTags,
}

/// Audio tags extracted from ID3 or other metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AudioTags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub composer: Option<String>,
    pub isrc: Option<String>,
}

/// Extract audio metadata from a file
pub async fn extract_audio_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
    let path = path.as_ref();

    // Read file for blake3 hash
    let file_data = tokio::fs::read(path).await
        .context("Failed to read audio file")?;

    let blake3_hash = blake3::hash(&file_data).to_hex().to_string();

    // Probe the file to detect format
    let tagged_file = Probe::open(path)
        .context("Failed to open audio file")?
        .read()
        .context("Failed to read audio metadata")?;

    // Extract properties
    let properties = tagged_file.properties();
    let duration_secs = properties.duration().as_secs_f64();
    let sample_rate = properties.sample_rate();
    let channels = properties.channels();
    let bitrate = properties.overall_bitrate();
    let format = format!("{:?}", tagged_file.file_type());

    // Extract tags
    let tags = if let Some(tag) = tagged_file.primary_tag() {
        AudioTags {
            title: tag.title().map(|s| s.to_string()),
            artist: tag.artist().map(|s| s.to_string()),
            album: tag.album().map(|s| s.to_string()),
            album_artist: tag.get_string(&ItemKey::AlbumArtist).map(|s| s.to_string()),
            track_number: tag.track().map(|n| n as u32),
            track_total: tag.track_total().map(|n| n as u32),
            disc_number: tag.disk().map(|n| n as u32),
            disc_total: tag.disk_total().map(|n| n as u32),
            year: tag.year().map(|y| y as i32),
            genre: tag.genre().map(|s| s.to_string()),
            comment: tag.comment().map(|s| s.to_string()),
            composer: tag.get_string(&ItemKey::Composer).map(|s| s.to_string()),
            isrc: tag.get_string(&ItemKey::Isrc).map(|s| s.to_string()),
        }
    } else {
        AudioTags::default()
    };

    Ok(AudioMetadata {
        duration_secs,
        sample_rate,
        channels,
        bitrate,
        format,
        blake3_hash,
        tags,
    })
}

/// Extract cover art from an audio file
pub async fn extract_cover_art<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
    let path = path.as_ref();

    let tagged_file = Probe::open(path)
        .context("Failed to open audio file")?
        .read()
        .context("Failed to read audio metadata")?;

    if let Some(tag) = tagged_file.primary_tag() {
        if let Some(picture) = tag.pictures().first() {
            return Ok(Some(picture.data().to_vec()));
        }
    }

    Ok(None)
}

/// Validate that a file is a supported audio format
pub fn is_audio_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".mp3")
        || lower.ends_with(".flac")
        || lower.ends_with(".m4a")
        || lower.ends_with(".aac")
        || lower.ends_with(".ogg")
        || lower.ends_with(".opus")
        || lower.ends_with(".wav")
        || lower.ends_with(".wma")
        || lower.ends_with(".ape")
        || lower.ends_with(".wv")
}

/// Validate that a file is a supported image format for artwork
pub fn is_image_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".webp")
        || lower.ends_with(".gif")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file("song.mp3"));
        assert!(is_audio_file("track.flac"));
        assert!(is_audio_file("audio.m4a"));
        assert!(is_audio_file("SONG.MP3"));
        assert!(!is_audio_file("video.mp4"));
        assert!(!is_audio_file("document.pdf"));
    }

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file("cover.jpg"));
        assert!(is_image_file("artwork.png"));
        assert!(is_image_file("COVER.JPEG"));
        assert!(!is_image_file("song.mp3"));
        assert!(!is_image_file("video.mp4"));
    }
}
