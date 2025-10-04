use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

/// Get the data directory path, creating it if it doesn't exist
pub fn get_data_dir() -> anyhow::Result<PathBuf> {
    let data_dir = PathBuf::from(".lens-node-data");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
        tracing::info!("Created data directory: {}", data_dir.display());
    }

    Ok(data_dir)
}

/// Save data to a JSON file
pub fn save_json<T: Serialize>(filename: &str, data: &T) -> anyhow::Result<()> {
    let data_dir = get_data_dir()?;
    let file_path = data_dir.join(filename);

    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(data)?;

    // Write atomically using a temporary file
    let temp_path = file_path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path)?;
    file.write_all(json.as_bytes())?;
    file.sync_all()?;

    // Rename to final location (atomic on Unix)
    fs::rename(&temp_path, &file_path)?;

    tracing::debug!("Saved data to {}", file_path.display());
    Ok(())
}

/// Load data from a JSON file
pub fn load_json<T: DeserializeOwned>(filename: &str) -> anyhow::Result<Option<T>> {
    let data_dir = get_data_dir()?;
    let file_path = data_dir.join(filename);

    if !file_path.exists() {
        tracing::debug!("No existing data file: {}", file_path.display());
        return Ok(None);
    }

    let json = fs::read_to_string(&file_path)?;
    let data = serde_json::from_str(&json)?;

    tracing::info!("Loaded data from {}", file_path.display());
    Ok(Some(data))
}

/// Check if a persistence file exists
pub fn file_exists(filename: &str) -> bool {
    if let Ok(data_dir) = get_data_dir() {
        data_dir.join(filename).exists()
    } else {
        false
    }
}
