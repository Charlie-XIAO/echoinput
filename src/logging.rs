use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use anyhow::{Context, Result};
use log::{Metadata, Record};

const MAX_LOG_BYTES: u64 = 1024 * 1024; // 1MB

/// Initialize the logger.
pub fn init() -> Result<()> {
    let (path, old_path) = ensure_log_file()?;

    let logger = Logger {
        path,
        old_path,
        lock: Mutex::new(()),
    };

    log::set_boxed_logger(Box::new(logger)).context("failed to initialize logger")?;
    log::set_max_level(log::STATIC_MAX_LEVEL);

    Ok(())
}

/// Open the log file with the default application.
pub fn open() -> Result<()> {
    let (path, _) = ensure_log_file()?;
    open::that(&path).with_context(|| format!("failed to open {}", path.display()))?;
    Ok(())
}

/// Ensure that the log file exists and return its path and its rotation path.
///
/// If the file does not yet exist, it will be created. The rotation path is
/// not guaranteed to exist.
fn ensure_log_file() -> Result<(PathBuf, PathBuf)> {
    let dir = dirs::data_local_dir()
        .context("failed to find log directory")?
        .join("echoinput");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create log directory: {}", dir.display()))?;

    let path = dir.join("echoinput.log");
    let old_path = dir.join("echoinput.old.log");
    if path.exists() {
        return Ok((path, old_path));
    }

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open log file {}", path.display()))?;
    Ok((path, old_path))
}

struct Logger {
    path: PathBuf,
    old_path: PathBuf,
    lock: Mutex<()>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::STATIC_MAX_LEVEL
    }

    fn log(&self, record: &Record<'_>) {
        let Ok(_guard) = self.lock.lock() else {
            return;
        };
        let _ = self.write(record);
    }

    fn flush(&self) {}
}

impl Logger {
    fn write(&self, record: &Record<'_>) -> Result<()> {
        self.rotate_if_needed()?;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        writeln!(
            file,
            "{} {:<5} {}: {}",
            humantime::format_rfc3339_seconds(SystemTime::now()),
            record.level(),
            record.target(),
            record.args()
        )?;

        Ok(())
    }

    fn rotate_if_needed(&self) -> Result<()> {
        let Ok(metadata) = std::fs::metadata(&self.path) else {
            return Ok(());
        };

        if metadata.len() < MAX_LOG_BYTES {
            return Ok(());
        }

        match std::fs::remove_file(&self.old_path) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
            Err(e) => return Err(e)?,
        }

        std::fs::rename(&self.path, &self.old_path)?;
        Ok(())
    }
}
