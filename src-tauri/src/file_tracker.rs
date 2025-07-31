use crate::config::Config;
use crate::error::FileTrackerError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

/// Represents a change in a file or directory.
#[derive(Debug, Serialize, Clone)]
pub enum FileChange {
    Created(PathBuf, FileMetadata),
    Modified(PathBuf, FileMetadata),
    Deleted(PathBuf),
}

impl std::fmt::Display for FileChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileChange::Created(path, _) => write!(f, "Novo: {}", path.display()),
            FileChange::Modified(path, _) => write!(f, "Modificado: {}", path.display()),
            FileChange::Deleted(path) => write!(f, "Deletado: {}", path.display()),
        }
    }
}

/// Metadata for a file or directory.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    last_modified: SystemTime,
    size: u64,
    is_dir: bool,
}

/// Tracks files in a directory and their metadata.
#[derive(Serialize, Deserialize)]
pub struct FileTracker {
    pub root_target: PathBuf,
    pub files_state: HashMap<PathBuf, FileMetadata>,
}

impl FileTracker {
    /// Creates a new FileTracker for the specified directory.
    pub fn new<T: AsRef<std::path::Path>>(root_target: T, config: &Config) -> Result<Self, FileTrackerError> {
        log::info!("Initializing FileTracker for directory: {}", root_target.as_ref().display());
        let root_target = root_target.as_ref();
        let files_state = Self::scan_dir(root_target)?;
        let file_tracker = FileTracker {
            files_state,
            root_target: root_target.to_path_buf(),
        };
        file_tracker.save(config)?;
        Ok(file_tracker)
    }

    /// Scans a directory and returns its file metadata.
    pub fn scan_dir<T: AsRef<std::path::Path>>(target: T) -> Result<HashMap<PathBuf, FileMetadata>, FileTrackerError> {
        let target = target.as_ref();
        let target_metadata = fs::metadata(target)?;

        // Check dir metadata
        if !target_metadata.is_dir() {
            return Err(FileTrackerError::NotADirectory);
        }

        let mut current_state = HashMap::new();
        for entry in WalkDir::new(target).follow_links(false) {
            let entry = entry?;
            let metadata = entry.metadata()?;
            current_state.insert(
                entry.into_path(),
                FileMetadata {
                    last_modified: metadata.modified()?,
                    size: metadata.len(),
                    is_dir: metadata.is_dir(),
                },
            );
        }
        Ok(current_state)
    }

    /// Computes differences between the current and previous file states.
    pub async fn diff(&mut self) -> Result<Vec<FileChange>, FileTrackerError> {
        let new_state = tokio::task::spawn_blocking({
            let target = self.root_target.clone();
            move || Self::scan_dir(target)
        })
        .await??;

        let mut changes = Vec::new();
        for (path, new_metadata) in &new_state {
            match self.files_state.get(path) {
                Some(old_metadata) => {
                    if old_metadata.last_modified != new_metadata.last_modified || old_metadata.size != new_metadata.size {
                        changes.push(FileChange::Modified(path.to_path_buf(), new_metadata.clone()));
                    }
                }
                None => changes.push(FileChange::Created(path.to_path_buf(), new_metadata.clone())),
            }
        }
        for path in self.files_state.keys() {
            if !new_state.contains_key(path) {
                changes.push(FileChange::Deleted(path.to_path_buf()));
            }
        }
        self.files_state = new_state;

        Ok(changes)
    }

    pub fn get_only_file_changes(all_changes: Vec<FileChange>) -> Vec<FileChange> {
        all_changes
            .into_iter()
            .filter_map(|element | {
                match &element {
                    FileChange::Created(_, metadata ) | 
                    FileChange::Modified(_, metadata ) => {
                        if !metadata.is_dir {
                            return Some(element)
                        }

                        None
                    },
                    FileChange::Deleted(_) => {
                        Some(element)
                    }
                }
            }).collect::<Vec<FileChange>>()
    }

    /// Saves the current state to the configured state file.
    pub fn save(&self, config: &Config) -> Result<(), FileTrackerError> {
        let mut file = File::create(&config.state_file_path)?;
        let json = serde_json::to_string_pretty(self)?;
        file.write(json.as_bytes())?;
        log::info!("Saved state to {}", config.state_file_path);
        Ok(())
    }

    /// Loads the FileTracker state from the configured state file.
    pub fn get(config: &Config) -> Result<Self, FileTrackerError> {
        let mut file = File::open(&config.state_file_path)?;
        let mut json_data = String::new();
        file.read_to_string(&mut json_data)?;
        Ok(serde_json::from_str(&json_data)?)
    }

    /// Stops monitoring and deletes the state file.
    pub fn stop_monitoring_and_delete_state(config: &Config) -> Result<(), FileTrackerError> {
        fs::remove_file(&config.state_file_path)?;
        log::info!("Stopped monitoring and deleted state file at {}", config.state_file_path);
        Ok(())
    }

    /// Checks if monitoring is active by checking the state file's existence.
    pub fn is_monitoring_active(config: &Config) -> bool {
        std::path::Path::new(&config.state_file_path).exists()
    }
}
