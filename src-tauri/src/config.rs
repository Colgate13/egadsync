use std::path::PathBuf;

/// Configuration module for the file monitoring application.
#[derive(Debug, Clone)]
pub struct Config {
    /// Interval for checking file changes (in seconds).
    pub sync_interval_secs: u64,
    /// Path to the state file for persisting FileTracker data.
    pub state_file_path: String,
}

impl Config {
    /// Gets the appropriate data directory for the application
    pub fn get_app_data_dir() -> Result<PathBuf, std::io::Error> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find data directory"))?;
        
        let app_dir = data_dir.join("egadsync");
        
        // Create the directory if it doesn't exist
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)?;
        }
        
        Ok(app_dir)
    }
    
    /// Creates a new Config with a secure state file path
    pub fn new() -> Result<Self, std::io::Error> {
        let app_data_dir = Self::get_app_data_dir()?;
        let state_file_path = app_data_dir.join("state.json");
        
        Ok(Config {
            sync_interval_secs: 60,
            state_file_path: state_file_path.to_string_lossy().to_string(),
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        // Use the secure configuration by default, fallback to current directory if it fails
        Self::new().unwrap_or_else(|_| Config {
            sync_interval_secs: 60,
            state_file_path: "./state.json".to_string(),
        })
    }
}
