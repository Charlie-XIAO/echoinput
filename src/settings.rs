use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer};

const MIN_HISTORY_LIMIT: usize = 1;
const MAX_HISTORY_LIMIT: usize = 10;
const DEFAULT_HISTORY_LIMIT: usize = 5;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(
        default = "default_history_limit",
        deserialize_with = "deserialize_history_limit"
    )]
    pub history_limit: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            history_limit: default_history_limit(),
        }
    }
}

const fn default_history_limit() -> usize {
    DEFAULT_HISTORY_LIMIT
}

fn deserialize_history_limit<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = usize::deserialize(deserializer)?;
    Ok(value.clamp(MIN_HISTORY_LIMIT, MAX_HISTORY_LIMIT))
}

/// Load settings from the settings file.
pub fn load() -> Result<Settings> {
    let path = ensure_settings_file()?;

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read settings from {}", path.display()))?;
    let settings = toml::from_str::<Settings>(&content)
        .with_context(|| format!("failed to parse settings from {}", path.display()))?;

    Ok(settings)
}

/// Open the settings file with the default application.
pub fn open() -> Result<()> {
    let path = ensure_settings_file()?;
    open::that(&path).with_context(|| format!("failed to open {}", path.display()))?;
    Ok(())
}

/// Ensure that the settings file exists and return its path.
///
/// If the file does not yet exist, it will be created with default content.
fn ensure_settings_file() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("failed to find settings directory")?
        .join("echoinput");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create settings directory: {}", dir.display()))?;

    let path = dir.join("settings.toml");
    if path.exists() {
        return Ok(path);
    }

    let writer = File::create(&path)
        .with_context(|| format!("failed to create settings file {}", path.display()))?;
    let mut writer = BufWriter::new(writer);
    write_default_settings(&mut writer)
        .with_context(|| format!("failed to write initial settings to {}", path.display()))?;

    Ok(path)
}

/// Write the default settings content.
#[rustfmt::skip]
fn write_default_settings<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "# Number of finalized keystroke rows to keep.")?;
    writeln!(w, "# Valid range: [{MIN_HISTORY_LIMIT}, {MAX_HISTORY_LIMIT}].")?;
    writeln!(w, "history_limit = {DEFAULT_HISTORY_LIMIT}")?;
    Ok(())
}
