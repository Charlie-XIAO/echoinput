use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};

use crate::keystrokes::KeystrokeVisibility;

const MIN_HISTORY_LIMIT: usize = 1;
const MAX_HISTORY_LIMIT: usize = 10;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    #[serde(deserialize_with = "deserialize_history_limit")]
    pub history_limit: usize,
    pub visibility: KeystrokeVisibility,
    pub placement: Placement,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            history_limit: 5,
            visibility: KeystrokeVisibility::default(),
            placement: Placement::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
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
            margin_x: 40,
            margin_y: 40,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
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

impl std::fmt::Display for PlacementAnchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TopLeft => f.write_str("Top left"),
            Self::TopRight => f.write_str("Top right"),
            Self::BottomLeft => f.write_str("Bottom left"),
            Self::BottomRight => f.write_str("Bottom right"),
        }
    }
}

#[derive(Debug)]
pub struct SettingsForm {
    pub history_limit: String,
    pub visibility: KeystrokeVisibility,
    pub anchor: PlacementAnchor,
    pub margin_x: String,
    pub margin_y: String,
}

/// A user interaction originating from the settings UI.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    HistoryLimitChanged(String),
    TypingVisibilityChanged(bool),
    ShortcutsVisibilityChanged(bool),
    SpecialKeysVisibilityChanged(bool),
    ModifierRowVisibilityChanged(bool),
    AnchorChanged(PlacementAnchor),
    MarginXChanged(String),
    MarginYChanged(String),
    Save,
    EditJson,
}

/// A side effect requested by a settings form interaction.
pub enum SettingsAction {
    Save(Settings),
    EditJson,
}

impl SettingsForm {
    pub fn new(settings: &Settings) -> Self {
        Self {
            history_limit: settings.history_limit.to_string(),
            visibility: settings.visibility,
            anchor: settings.placement.anchor,
            margin_x: settings.placement.margin_x.to_string(),
            margin_y: settings.placement.margin_y.to_string(),
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Option<SettingsAction> {
        match message {
            SettingsMessage::HistoryLimitChanged(value) => self.history_limit = value,
            SettingsMessage::TypingVisibilityChanged(value) => self.visibility.typing = value,
            SettingsMessage::ShortcutsVisibilityChanged(value) => self.visibility.shortcuts = value,
            SettingsMessage::SpecialKeysVisibilityChanged(value) => {
                self.visibility.special_keys = value
            },
            SettingsMessage::ModifierRowVisibilityChanged(value) => {
                self.visibility.modifier_row = value
            },
            SettingsMessage::AnchorChanged(value) => self.anchor = value,
            SettingsMessage::MarginXChanged(value) => self.margin_x = value,
            SettingsMessage::MarginYChanged(value) => self.margin_y = value,
            SettingsMessage::Save => return self.settings().map(SettingsAction::Save),
            SettingsMessage::EditJson => return Some(SettingsAction::EditJson),
        }
        None
    }

    pub fn settings(&self) -> Option<Settings> {
        if !self.history_limit_is_valid() || !self.margin_x_is_valid() || !self.margin_y_is_valid()
        {
            return None;
        }

        Some(Settings {
            history_limit: self.history_limit.parse().ok()?,
            visibility: self.visibility,
            placement: Placement {
                anchor: self.anchor,
                margin_x: self.margin_x.parse().ok()?,
                margin_y: self.margin_y.parse().ok()?,
            },
        })
    }

    pub fn history_limit_is_valid(&self) -> bool {
        self.history_limit
            .parse()
            .is_ok_and(|value| (MIN_HISTORY_LIMIT..=MAX_HISTORY_LIMIT).contains(&value))
    }

    pub fn margin_x_is_valid(&self) -> bool {
        self.margin_x.parse::<u32>().is_ok()
    }

    pub fn margin_y_is_valid(&self) -> bool {
        self.margin_y.parse::<u32>().is_ok()
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

    let file = File::open(&path)
        .with_context(|| format!("failed to open settings file: {}", path.display()))?;
    let reader = BufReader::new(file);
    let settings = serde_json::from_reader(reader)
        .with_context(|| format!("failed to parse settings file: {}", path.display()))?;

    Ok(settings)
}

/// Save settings to the settings file.
pub fn save(settings: &Settings) -> Result<()> {
    let path = ensure_settings_file()?;
    write_settings(&path, settings)
}

/// Open the settings file with the default application.
pub fn open() -> Result<()> {
    let path = ensure_settings_file()?;
    open::that(&path).with_context(|| format!("failed to open {}", path.display()))?;
    Ok(())
}

/// Ensure that the settings file exists and return its path.
fn ensure_settings_file() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("failed to find settings directory")?
        .join("echoinput");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create settings directory: {}", dir.display()))?;

    let path = dir.join("settings.json");
    if !path.exists() {
        std::fs::write(&path, "{}")
            .with_context(|| format!("failed to create empty settings file: {}", path.display()))?;
    }
    Ok(path)
}

fn write_settings(path: &Path, settings: &Settings) -> Result<()> {
    let file = File::create(path)
        .with_context(|| format!("failed to create settings file: {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, settings)
        .with_context(|| format!("failed to write settings file: {}", path.display()))?;
    Ok(())
}
