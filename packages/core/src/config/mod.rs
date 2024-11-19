pub mod core_config;
pub mod manager;

use std::path::PathBuf;

use log::debug;
use manager::ConfigManager;

/**
 * Initialize configuration
 */
pub fn init_config(path: &PathBuf) -> ConfigManager {
    let path_display = path.display().to_string();

    debug!(
        "Initializing config file, provided location : {}",
        path_display
    );

    let config_path = path.join(".bpm");

    let config_manager = ConfigManager::from(&config_path);

    debug!(
        "Done initializing config file using location {} !",
        path_display
    );

    config_manager
}

#[cfg(test)]
mod tests {

    use super::*;
}
