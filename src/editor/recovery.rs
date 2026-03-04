use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Interval between auto-save recovery writes.
const RECOVERY_INTERVAL: Duration = Duration::from_secs(30);

/// Manages periodic auto-save snapshots so that unsaved work can be recovered
/// after a crash.  The recovery file is written alongside the original file
/// (or in a temp directory when no file path is set), with a `.recovery`
/// extension appended.
pub struct RecoveryManager {
    recovery_path: PathBuf,
    last_save: Option<Instant>,
}

impl RecoveryManager {
    /// Construct a manager that writes recovery files to `recovery_dir`.
    pub fn new(recovery_dir: impl Into<PathBuf>) -> Self {
        Self {
            recovery_path: recovery_dir.into(),
            last_save: None,
        }
    }

    /// Construct a manager that derives the recovery path from the document
    /// file path by appending `.recovery`.
    pub fn for_file(file_path: &Path) -> Self {
        let mut recovery_path = file_path.to_path_buf();
        let mut name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        name.push_str(".recovery");
        recovery_path.set_file_name(name);
        Self {
            recovery_path,
            last_save: None,
        }
    }

    /// Called on every application tick.  Writes a recovery snapshot if the
    /// buffer is dirty and enough time has elapsed.
    pub fn tick(&mut self, content: &str, is_dirty: bool) {
        if !is_dirty {
            return;
        }
        let now = Instant::now();
        let due = self
            .last_save
            .map(|t| now.duration_since(t) >= RECOVERY_INTERVAL)
            .unwrap_or(true);

        if due {
            let _ = self.write_snapshot(content);
            self.last_save = Some(now);
        }
    }

    /// Check whether a recovery file exists.  Returns its path when found.
    pub fn check_recovery(&self) -> Option<&Path> {
        if self.recovery_path.exists() {
            Some(&self.recovery_path)
        } else {
            None
        }
    }

    /// Load the recovery file contents.
    pub fn load_recovery(&self) -> std::io::Result<String> {
        std::fs::read_to_string(&self.recovery_path)
    }

    /// Remove the recovery file after a successful save.
    pub fn clear_recovery(&self) -> std::io::Result<()> {
        if self.recovery_path.exists() {
            std::fs::remove_file(&self.recovery_path)?;
        }
        Ok(())
    }

    fn write_snapshot(&self, content: &str) -> std::io::Result<()> {
        if let Some(parent) = self.recovery_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.recovery_path, content)
    }
}
