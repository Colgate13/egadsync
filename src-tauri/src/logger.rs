use log::LevelFilter;
use env_logger::Builder;

/// Initializes the logging system.
pub fn init() {
    Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
}
