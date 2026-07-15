use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer};

const MIN_HISTORY_LIMIT: usize = 1;
const MAX_HISTORY_LIMIT: usize = 10;
const DEFAULT_HISTORY_LIMIT: usize = 5;

const DEFAULT_MARGIN: u32 = 40;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    #[serde(deserialize_with = "deserialize_history_limit")]
    pub history_limit: usize,
    pub placement: Placement,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            history_limit: DEFAULT_HISTORY_LIMIT,
            placement: Placement::default(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Placement {
    pub anchor: PlacementAnchor,
    pub margin_x: u32,
    pub margin_y: u32,
}

impl Default for Placement {
    fn default() -> Self {
        Self {
            anchor: PlacementAnchor::BottomLeft,
            margin_x: DEFAULT_MARGIN,
            margin_y: DEFAULT_MARGIN,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PlacementAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl PlacementAnchor {
    pub fn is_top(&self) -> bool {
        matches!(self, Self::TopLeft | Self::TopRight)
    }

    pub fn is_right(&self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }
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
    writeln!(w, "# Maximum number of finalized keystroke rows to keep.")?;
    writeln!(w, "# Valid range: [{MIN_HISTORY_LIMIT}, {MAX_HISTORY_LIMIT}].")?;
    writeln!(w, "history_limit = {DEFAULT_HISTORY_LIMIT}")?;
    writeln!(w)?;
    writeln!(w, "[placement]")?;
    writeln!(w)?;
    writeln!(w, "# The corner of the screen to which the overlay is anchored.")?;
    writeln!(w, "# If anchored top, the layout will be top to bottom.")?;
    writeln!(w, "# If anchored bottom, the layout will be bottom to top.")?;
    writeln!(w, "# Valid values: bottom-left, bottom-right, top-left, top-right.")?;
    writeln!(w, "anchor = \"bottom-left\"")?;
    writeln!(w)?;
    writeln!(w, "# Margins from the anchored corner in pixels.")?;
    writeln!(w, "margin_x = {DEFAULT_MARGIN}")?;
    writeln!(w, "margin_y = {DEFAULT_MARGIN}")?;
    Ok(())
}
