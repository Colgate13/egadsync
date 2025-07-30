use serde::{ser::SerializeStruct, Serializer};
use std::error::Error;
use std::fmt;
use std::io;

/// Custom error type for the file monitoring application.
#[derive(Debug)]
pub enum FileTrackerError {
    NotADirectory,
    IoError(io::Error),
    WalkdirError(walkdir::Error),
    JoinError(tokio::task::JoinError),
    SerdeJsonError(serde_json::Error),
}

impl Error for FileTrackerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FileTrackerError::NotADirectory => None,
            FileTrackerError::IoError(err) => Some(err),
            FileTrackerError::WalkdirError(err) => Some(err),
            FileTrackerError::JoinError(err) => Some(err),
            FileTrackerError::SerdeJsonError(err) => Some(err),
        }
    }
}

impl fmt::Display for FileTrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileTrackerError::NotADirectory => write!(f, "The specified path is not a directory"),
            FileTrackerError::IoError(err) => write!(f, "I/O error: {}", err),
            FileTrackerError::WalkdirError(err) => write!(f, "File scanning error: {}", err),
            FileTrackerError::JoinError(err) => write!(f, "Background task error: {}", err),
            FileTrackerError::SerdeJsonError(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl serde::Serialize for FileTrackerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FileTrackerError", 2)?;
        match self {
            FileTrackerError::NotADirectory => {
                state.serialize_field("type", "NotADirectory")?;
                state.serialize_field("details", "The specified path is not a directory")?;
            }
            FileTrackerError::IoError(err) => {
                state.serialize_field("type", "IoError")?;
                state.serialize_field("details", &err.to_string())?;
            }
            FileTrackerError::WalkdirError(err) => {
                state.serialize_field("type", "WalkdirError")?;
                state.serialize_field("details", &err.to_string())?;
            }
            FileTrackerError::JoinError(err) => {
                state.serialize_field("type", "JoinError")?;
                state.serialize_field("details", &err.to_string())?;
            }
            FileTrackerError::SerdeJsonError(err) => {
                state.serialize_field("type", "SerdeJsonError")?;
                state.serialize_field("details", &err.to_string())?;
            }
        }
        state.end()
    }
}

impl From<io::Error> for FileTrackerError {
    fn from(err: io::Error) -> Self {
        FileTrackerError::IoError(err)
    }
}

impl From<walkdir::Error> for FileTrackerError {
    fn from(err: walkdir::Error) -> Self {
        FileTrackerError::WalkdirError(err)
    }
}

impl From<tokio::task::JoinError> for FileTrackerError {
    fn from(err: tokio::task::JoinError) -> Self {
        FileTrackerError::JoinError(err)
    }
}

impl From<serde_json::Error> for FileTrackerError {
    fn from(err: serde_json::Error) -> Self {
        FileTrackerError::SerdeJsonError(err)
    }
}
