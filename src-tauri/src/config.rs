/// Configuration module for the file monitoring application.
#[derive(Debug, Clone)]
pub struct Config {
    /// Interval for checking file changes (in seconds).
    pub sync_interval_secs: u64,
    /// Path to the state file for persisting FileTracker data.
    pub state_file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sync_interval_secs: 60,
            state_file_path: "./state.json".to_string(),
        }
    }
}
