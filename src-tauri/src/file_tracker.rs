use serde::{ser::SerializeStruct, Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self},
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
    time::SystemTime,
};
use tokio::task::{self, JoinError};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum FileTrackerError {
    IsNotDir,
    IoError(io::Error),
    JoinErrorTask(JoinError),
    WaldirError(walkdir::Error),
    SerdeJson(serde_json::Error),
}

#[derive(Debug, Serialize, Clone)]
pub enum FileChange {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

impl Error for FileTrackerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FileTrackerError::IsNotDir => None,
            FileTrackerError::IoError(err) => Some(err),
            FileTrackerError::WaldirError(err) => Some(err),
            FileTrackerError::JoinErrorTask(err) => Some(err),
            FileTrackerError::SerdeJson(err) => Some(err),
        }
    }
}

impl fmt::Display for FileTrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileTrackerError::IsNotDir => {
                write!(f, "A pasta inicial não é um diretorio!")
            }
            FileTrackerError::IoError(err) => {
                write!(f, "Erro de I/O: {}", err)
            }
            FileTrackerError::WaldirError(err) => {
                write!(f, "Error no scanner de arquivos: {}", err)
            }
            FileTrackerError::JoinErrorTask(err) => {
                write!(f, "Error para executar tarefas em background: {}", err)
            }
            FileTrackerError::SerdeJson(err) => {
                write!(f, "Error para executar tarefas em background: {}", err)
            }
        }
    }
}

impl Serialize for FileTrackerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        let mut state = serializer.serialize_struct("FileTrackerError", 2)?;
        match self {
            FileTrackerError::IoError(err) => {
                state.serialize_field("type", "IsNotDir")?;
                state.serialize_field("details", &format!("{}", err))?;
            },
            _ => {
                state.serialize_field("type", "InternalError")?;
                state.serialize_field("details", &format!("Internal Error"))?;
            }
        };
        
        state.end()
    }
}

impl From<io::Error> for FileTrackerError {
    fn from(value: io::Error) -> Self {
        FileTrackerError::IoError(value)
    }
}

impl From<walkdir::Error> for FileTrackerError {
    fn from(value: walkdir::Error) -> Self {
        FileTrackerError::WaldirError(value)
    }
}

impl From<JoinError> for FileTrackerError {
    fn from(value: JoinError) -> Self {
        FileTrackerError::JoinErrorTask(value)
    }
}

impl From<serde_json::Error> for FileTrackerError {
    fn from(value: serde_json::Error) -> Self {
        FileTrackerError::SerdeJson(value)
    }
}


impl fmt::Display for FileChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileChange::Created(path) => write!(f, "Created: {}", path.display()),
            FileChange::Modified(path) => write!(f, "Modified: {}", path.display()),
            FileChange::Deleted(path) => write!(f, "Deleted: {}", path.display()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    last_modified: SystemTime,
    size: u64,
    is_dir: bool,
}

type StateHashMap = HashMap<PathBuf, FileMetadata>;

#[derive(Serialize, Deserialize)]
pub struct FileTracker {
    root_target: PathBuf,
    files_state: StateHashMap,
}

impl FileTracker {
    pub fn new<T: AsRef<std::path::Path>>(root_target: T) -> Result<Self, FileTrackerError> {
        println!("Initialize new state");
        let root_target = root_target.as_ref();
        let files_state = FileTracker::scan_dir(root_target)?;
        let file_tracker = FileTracker {
            files_state,
            root_target: root_target.to_path_buf(),
        };
        file_tracker.save()?;

        Ok(file_tracker)
    }

    pub fn scan_dir<T: AsRef<std::path::Path>>(
        target: T,
    ) -> Result<StateHashMap, FileTrackerError> {
        let target = target.as_ref();
        let target_metadata = fs::metadata(&target)?;

        if !target_metadata.is_dir() {
            return Err(FileTrackerError::IsNotDir);
        }

        let setup_walkdir = WalkDir::new(target).follow_links(false);

        let mut current_state: StateHashMap = HashMap::new();
        for entry in setup_walkdir {
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

    pub async fn diff(&mut self) -> Result<Vec<FileChange>, FileTrackerError> {
        let new_state = task::spawn_blocking({
            let target = self.root_target.clone();
            move || Self::scan_dir(target)
        })
        .await??;
        let mut changes: Vec<FileChange> = Vec::new();

        // Created or Modified files.
        for (path, new_metadata) in &new_state {
            match self.files_state.get(path) {
                Some(old_metadata) => {
                    // Check metadata to detect changes
                    if old_metadata.last_modified != new_metadata.last_modified
                        || old_metadata.size != new_metadata.size
                    {
                        changes.push(FileChange::Modified(path.to_path_buf()));
                    }
                }
                None => {
                    // New file
                    changes.push(FileChange::Created(path.to_path_buf()));
                }
            }
        }

        // Deleted file
        for path in self.files_state.keys() {
            if !new_state.contains_key(path) {
                changes.push(FileChange::Deleted(path.to_path_buf()));
            }
        }

        // Update file state to new state.
        self.files_state = new_state;

        Ok(changes)
    }

    pub fn save(&self) -> Result<(), FileTrackerError> {
        let mut file = File::create("./state.json")?;
        let json = serde_json::to_string_pretty(&self)?;
        file.write(json.as_bytes())?;

        Ok(())
    }

    pub fn get() -> Result<FileTracker, FileTrackerError> {
        let mut file = File::open("./state.json")?;
        let mut json_data = String::new();
        file.read_to_string(&mut json_data)?;

        Ok(serde_json::from_str::<FileTracker>(&json_data)?)
    }
}
