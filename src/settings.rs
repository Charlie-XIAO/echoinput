use std::path::PathBuf;
use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::keystrokes::DEFAULT_HISTORY_LIMIT;

pub const MIN_HISTORY_LIMIT: usize = 1;
pub const MAX_HISTORY_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            history_limit: DEFAULT_HISTORY_LIMIT,
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        self.history_limit = self
            .history_limit
            .clamp(MIN_HISTORY_LIMIT, MAX_HISTORY_LIMIT);
        self
    }
}

pub fn load() -> AppSettings {
    let Some(path) = config_path() else {
        eprintln!("Could not find a config directory; using default settings.");
        return AppSettings::default();
    };

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
            Ok(settings) => settings.normalized(),
            Err(err) => {
                eprintln!("Could not parse settings at {}: {err}", path.display());
                AppSettings::default()
            },
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => AppSettings::default(),
        Err(err) => {
            eprintln!("Could not read settings at {}: {err}", path.display());
            AppSettings::default()
        },
    }
}

pub fn save(settings: AppSettings) {
    if let Err(err) = try_save(settings.normalized()) {
        eprintln!("Could not save settings: {err}");
    }
}

fn try_save(settings: AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "could not find a config directory")
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(path, content)?;

    Ok(())
}

fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("echoinput").join("settings.json"))
}

fn default_history_limit() -> usize {
    DEFAULT_HISTORY_LIMIT
}
